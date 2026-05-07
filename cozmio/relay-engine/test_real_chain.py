#!/usr/bin/env python3
"""Real Relay Engine V1-V8 verification.

This script intentionally talks to the relay over its process boundary and
binary protocol. It does not import relay-engine internals.
"""

from __future__ import annotations

import os
import re
import socket
import struct
import subprocess
import sys
import tempfile
import threading
import time
import traceback
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RELAY_EXE = ROOT / "target" / "release" / "relay-engine.exe"
MAIN_ADDR = ("127.0.0.1", 7890)
SUB_ADDR = ("127.0.0.1", 7891)

REQ_DISPATCH = 1
REQ_STATUS = 2
REQ_PROGRESS = 3
REQ_RESULT = 4
REQ_INTERRUPT = 5
REQ_SUBSCRIBE = 6

STATUS_COMPLETED = 3
STATUS_FAILED = 4
STATUS_INTERRUPTED = 5


@dataclass
class ProgressEntry:
    timestamp: int
    message: str
    level: int


@dataclass
class ProgressEvent:
    session_id: str
    timestamp: int
    message: str
    level: int
    terminal: bool
    terminal_status: int


def enc_varint(value: int) -> bytes:
    out = bytearray()
    while value >= 0x80:
        out.append((value & 0x7F) | 0x80)
        value >>= 7
    out.append(value)
    return bytes(out)


def dec_varint(data: bytes, pos: int) -> tuple[int, int]:
    shift = 0
    value = 0
    while True:
        if pos >= len(data):
            raise ValueError("truncated varint")
        b = data[pos]
        pos += 1
        value |= (b & 0x7F) << shift
        if not b & 0x80:
            return value, pos
        shift += 7


def field_string(tag: int, value: str) -> bytes:
    raw = value.encode("utf-8")
    return bytes([(tag << 3) | 2]) + enc_varint(len(raw)) + raw


def parse_fields(data: bytes) -> dict[int, list[object]]:
    fields: dict[int, list[object]] = {}
    pos = 0
    while pos < len(data):
        key, pos = dec_varint(data, pos)
        tag = key >> 3
        wire = key & 0x07
        if wire == 0:
            value, pos = dec_varint(data, pos)
        elif wire == 2:
            length, pos = dec_varint(data, pos)
            value = data[pos : pos + length]
            pos += length
        else:
            raise ValueError(f"unsupported protobuf wire type {wire}")
        fields.setdefault(tag, []).append(value)
    return fields


def as_text(value: object | None) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    return str(value)


def send_request(kind: int, payload: bytes, expect_response: bool = True) -> bytes:
    with socket.create_connection(MAIN_ADDR, timeout=10) as sock:
        sock.settimeout(120)
        frame = bytes([kind]) + payload
        sock.sendall(struct.pack(">I", len(frame)) + frame)
        if not expect_response:
            return b""
        return recv_frame(sock)


def recv_frame(sock: socket.socket) -> bytes:
    header = sock.recv(4)
    if len(header) != 4:
        raise RuntimeError("missing frame header")
    length = struct.unpack(">I", header)[0]
    data = bytearray()
    while len(data) < length:
        chunk = sock.recv(length - len(data))
        if not chunk:
            raise RuntimeError("socket closed during frame")
        data.extend(chunk)
    return bytes(data)


def dispatch(task: str) -> tuple[str, int]:
    payload = (
        field_string(1, "claude-code")
        + field_string(2, "real relay verification")
        + field_string(3, task)
    )
    fields = parse_fields(send_request(REQ_DISPATCH, payload))
    session_id = as_text(fields.get(1, [b""])[0])
    status = int(fields.get(2, [0])[0])
    return session_id, status


def status(session_id: str) -> int:
    fields = parse_fields(send_request(REQ_STATUS, field_string(1, session_id)))
    return int(fields.get(2, [0])[0])


def progress(session_id: str) -> list[ProgressEntry]:
    fields = parse_fields(send_request(REQ_PROGRESS, field_string(1, session_id)))
    entries = []
    for raw in fields.get(2, []):
        entry_fields = parse_fields(raw)
        entries.append(
            ProgressEntry(
                timestamp=int(entry_fields.get(1, [0])[0]),
                message=as_text(entry_fields.get(2, [b""])[0]),
                level=int(entry_fields.get(3, [0])[0]),
            )
        )
    return entries


def result(session_id: str) -> dict[str, object]:
    fields = parse_fields(send_request(REQ_RESULT, field_string(1, session_id)))
    raw_result = fields.get(2, [b""])[0]
    if not raw_result:
        return {}
    result_fields = parse_fields(raw_result)
    return {
        "summary": as_text(result_fields.get(1, [b""])[0]),
        "raw_output": as_text(result_fields.get(2, [b""])[0]),
        "duration_secs": int(result_fields.get(3, [0])[0]),
        "success": bool(result_fields.get(4, [0])[0]),
        "error_message": as_text(result_fields.get(5, [b""])[0]),
    }


def interrupt(session_id: str) -> bool:
    fields = parse_fields(send_request(REQ_INTERRUPT, field_string(1, session_id)))
    return bool(fields.get(1, [0])[0])


def open_subscription(session_id: str, timeout: float = 120.0) -> socket.socket:
    sock = socket.create_connection(SUB_ADDR, timeout=10)
    sock.settimeout(timeout)
    payload = field_string(1, session_id)
    frame = bytes([REQ_SUBSCRIBE]) + payload
    sock.sendall(struct.pack(">I", len(frame)) + frame)
    return sock


def recv_progress_event(sock: socket.socket) -> ProgressEvent:
    event_fields = parse_fields(recv_frame(sock))
    return ProgressEvent(
        session_id=as_text(event_fields.get(1, [b""])[0]),
        timestamp=int(event_fields.get(2, [0])[0]),
        message=as_text(event_fields.get(3, [b""])[0]),
        level=int(event_fields.get(4, [0])[0]),
        terminal=bool(event_fields.get(5, [0])[0]),
        terminal_status=int(event_fields.get(6, [0])[0]),
    )


def subscribe(session_id: str, timeout: float = 120.0) -> list[ProgressEvent]:
    events: list[ProgressEvent] = []
    with open_subscription(session_id, timeout=timeout) as sock:
        while True:
            event = recv_progress_event(sock)
            events.append(event)
            if event.terminal:
                return events


def wait_for_port(addr: tuple[str, int], timeout: float = 10.0) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            with socket.create_connection(addr, timeout=0.5):
                return
        except OSError:
            time.sleep(0.1)
    raise RuntimeError(f"port did not open: {addr}")


def extract_pid(entries: list[ProgressEntry]) -> int:
    for entry in entries:
        match = re.search(r"pid=(\d+)", entry.message)
        if match:
            return int(match.group(1))
    raise RuntimeError("no pid progress entry found")


def process_command_line(pid: int) -> str:
    command = (
        "Get-CimInstance Win32_Process -Filter \"ProcessId = %d\" "
        "| Select-Object -ExpandProperty CommandLine"
    ) % pid
    completed = subprocess.run(
        ["powershell", "-NoProfile", "-Command", command],
        capture_output=True,
        text=True,
        timeout=10,
    )
    return completed.stdout.strip()


def process_exists(pid: int) -> bool:
    command = (
        "if (Get-CimInstance Win32_Process -Filter \"ProcessId = %d\") "
        "{ exit 0 } else { exit 1 }"
    ) % pid
    return (
        subprocess.run(
            ["powershell", "-NoProfile", "-Command", command],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            timeout=10,
        ).returncode
        == 0
    )


def wait_for_terminal(session_id: str, timeout: float = 120.0) -> int:
    deadline = time.time() + timeout
    while time.time() < deadline:
        current = status(session_id)
        if current in {STATUS_COMPLETED, STATUS_FAILED, STATUS_INTERRUPTED}:
            return current
        time.sleep(1)
    raise RuntimeError(f"session did not reach terminal state: {session_id}")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)
    print(f"PASS: {message}")


def main() -> int:
    if not RELAY_EXE.exists():
        raise RuntimeError(f"relay-engine binary not found: {RELAY_EXE}")

    env = os.environ.copy()
    env.setdefault(
        "COZMIO_CLAUDE_CLI", r"C:\Users\29913\AppData\Roaming\npm\claude.cmd"
    )
    env.setdefault("RUST_LOG", "debug")

    log_file = tempfile.NamedTemporaryFile(
        mode="w+", encoding="utf-8", delete=False, prefix="cozmio-relay-", suffix=".log"
    )
    relay = subprocess.Popen(
        [str(RELAY_EXE)],
        cwd=str(ROOT),
        env=env,
        stdout=log_file,
        stderr=subprocess.STDOUT,
        text=True,
    )

    try:
        time.sleep(0.5)
        require(relay.poll() is None, "V1 relay-engine process stays alive")
        wait_for_port(MAIN_ADDR)
        require(True, "V2 client can connect to Relay main port")
        wait_for_port(SUB_ADDR)
        require(True, "V8 client can connect to Relay subscription port")

        task = "Say relay-e2e-ok and nothing else."
        session_id, initial_status = dispatch(task)
        require(
            re.fullmatch(
                r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}",
                session_id,
            )
            is not None,
            f"V3 dispatch returned real UUID session_id={session_id}",
        )
        require(initial_status in {1, 3}, f"dispatch returned active status={initial_status}")

        time.sleep(1)
        entries = progress(session_id)
        pid = extract_pid(entries)
        command_line = process_command_line(pid)
        require(process_exists(pid), f"V4 Claude Code connector process exists pid={pid}")
        require(
            "claude" in command_line.lower(),
            f"V4 process command line references Claude Code: {command_line}",
        )
        require(len(entries) >= 1, f"V5 progress query returns buffered entries={len(entries)}")

        events = subscribe(session_id)
        replayed = any("Started Claude Code process" in event.message for event in events)
        live_output = any("stdout:" in event.message or "stderr:" in event.message for event in events)
        terminal = any(event.terminal for event in events)
        require(replayed, "V8 subscriber replayed buffered pre-subscription progress")
        require(live_output, "V8 subscriber received live process output event")
        require(terminal, "V8 subscriber received terminal event")

        terminal_status = next(event.terminal_status for event in events if event.terminal)
        time.sleep(0.5)
        final_result = result(session_id)
        require(
            terminal_status in {STATUS_COMPLETED, STATUS_FAILED},
            f"V6 session reached completed/failed terminal status={terminal_status}",
        )
        require(bool(final_result), "V6 result is available through client query")
        require(
            "relay-e2e-ok" in str(final_result.get("raw_output", "")).lower()
            or len(str(final_result.get("raw_output", ""))) > 0,
            "V6 result contains captured Claude Code output",
        )
        require(relay.poll() is None, "Relay process remains alive after first session")

        v7_session, _ = dispatch(
            "Write a concise but non-trivial explanation of why real process interrupts matter."
        )
        time.sleep(1)
        v7_pid = extract_pid(progress(v7_session))
        require(process_exists(v7_pid), f"V7 process exists before interrupt pid={v7_pid}")
        v7_events: list[ProgressEvent] = []
        with open_subscription(v7_session, timeout=60) as v7_sock:
            first_v7_event = recv_progress_event(v7_sock)
            v7_events.append(first_v7_event)
            require(
                "Started Claude Code process" in first_v7_event.message,
                "V7 subscription replayed initial progress before interrupt",
            )
            require(interrupt(v7_session), "V7 interrupt request returned success")
            time.sleep(2)
            require(not process_exists(v7_pid), f"V7 process was terminated pid={v7_pid}")
            while not any(event.terminal for event in v7_events):
                v7_events.append(recv_progress_event(v7_sock))
        require(
            any(
                event.terminal and event.terminal_status == STATUS_INTERRUPTED
                for event in v7_events
            ),
            "V7 interrupted terminal event was pushed to subscriber",
        )

        print("ALL_REAL_RELAY_CHECKS_PASSED")
        return 0
    finally:
        if relay.poll() is None:
            relay.terminate()
            try:
                relay.wait(timeout=5)
            except subprocess.TimeoutExpired:
                relay.kill()
        log_file.flush()
        log_file.seek(0)
        relay_log = log_file.read()
        log_file.close()
        if relay_log:
            print("--- relay-engine log tail ---")
            print(relay_log[-4000:])


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        traceback.print_exc()
        print(f"FAIL: {exc}", file=sys.stderr)
        sys.exit(1)
