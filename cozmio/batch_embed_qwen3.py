"""
Real Memory Embedding Integration
Uses Qwen3-Embedding-0.6B via sentence-transformers Route A
Generates 1024-dim embeddings for imported events and context_slices
Stores in SQLite BLOB + metadata
"""

import sqlite3
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

from sentence_transformers import SentenceTransformer
import numpy as np

# Paths
DB_PATH = "C:/Users/29913/AppData/Local/cozmio/memory/memory.db"
MODEL_PATH = "D:/tmp/hf_cache_qwen3/models--Qwen--Qwen3-Embedding-0.6B/snapshots/97b0c614be4d77ee51c0cef4e5f07c00f9eb65b3/"

MODEL_NAME = "Qwen/Qwen3-Embedding-0.6B"
DIM = 1024

def cosine_sim(a, b):
    return float(np.dot(a, b))

def migrate_db():
    conn = sqlite3.connect(DB_PATH)
    cur = conn.cursor()

    # Check if embedding column exists in context_slices
    cur.execute("PRAGMA table_info(context_slices)")
    cols = [row[1] for row in cur.fetchall()]
    if 'embedding' not in cols:
        print("Adding embedding column to context_slices...")
        cur.execute("ALTER TABLE context_slices ADD COLUMN embedding BLOB")
        conn.commit()
        print("  Done.")

    # Add metadata columns to memory_events if not exist
    cur.execute("PRAGMA table_info(memory_events)")
    cols = [row[1] for row in cur.fetchall()]
    if 'embedding_model' not in cols:
        print("Adding metadata columns to memory_events...")
        cur.execute("ALTER TABLE memory_events ADD COLUMN embedding_model TEXT")
        cur.execute("ALTER TABLE memory_events ADD COLUMN embedding_dimension INTEGER")
        cur.execute("ALTER TABLE memory_events ADD COLUMN embedding_generated_at TEXT")
        cur.execute("ALTER TABLE memory_events ADD COLUMN embedding_text TEXT")
        conn.commit()
        print("  Done.")

    # Add metadata columns to context_slices if not exist
    cur.execute("PRAGMA table_info(context_slices)")
    cols = [row[1] for row in cur.fetchall()]
    if 'embedding_model' not in cols:
        print("Adding metadata columns to context_slices...")
        cur.execute("ALTER TABLE context_slices ADD COLUMN embedding_model TEXT")
        cur.execute("ALTER TABLE context_slices ADD COLUMN embedding_dimension INTEGER")
        cur.execute("ALTER TABLE context_slices ADD COLUMN embedding_generated_at TEXT")
        cur.execute("ALTER TABLE context_slices ADD COLUMN embedding_text TEXT")
        conn.commit()
        print("  Done.")

    conn.close()

def load_model():
    print(f"Loading Qwen3-Embedding-0.6B from: {MODEL_PATH}")
    model = SentenceTransformer(MODEL_PATH)
    dim = model.get_sentence_embedding_dimension()
    print(f"  Model loaded. Embedding dimension: {dim}")
    return model

def generate_embeddings(model, texts, batch_size=32):
    print(f"  Generating embeddings for {len(texts)} texts...")
    embeddings = model.encode(texts, normalize_embeddings=True, batch_size=batch_size, show_progress_bar=True)
    print(f"  Done. Shape: {embeddings.shape}")
    return embeddings

def embed_memory_events(model, conn):
    cur = conn.cursor()

    cur.execute("""
        SELECT id, timestamp, content, window_title, source, evidence_source
        FROM memory_events
        WHERE embedding IS NULL AND evidence_source = 'imported'
        ORDER BY timestamp
    """)
    rows = cur.fetchall()
    print(f"\n=== Embedding {len(rows)} memory_events ===")
    if len(rows) == 0:
        print("  No events need embedding.")
        return 0

    ids = [r[0] for r in rows]
    texts = [r[2] for r in rows]

    embeddings = generate_embeddings(model, texts)
    generated_at = datetime.now(timezone.utc).isoformat()

    count = 0
    for i, (event_id, text) in enumerate(zip(ids, texts)):
        emb = embeddings[i]
        emb_bytes = emb.astype(np.float32).tobytes()
        cur.execute("""
            UPDATE memory_events
            SET embedding = ?,
                embedding_model = ?,
                embedding_dimension = ?,
                embedding_generated_at = ?,
                embedding_text = ?
            WHERE id = ?
        """, (emb_bytes, MODEL_NAME, DIM, generated_at, text, event_id))
        count += 1

    conn.commit()
    print(f"  Embedded {count} memory_events")
    return count

def embed_context_slices(model, conn):
    cur = conn.cursor()

    cur.execute("""
        SELECT id, summary, topics, evidence_source
        FROM context_slices
        WHERE embedding IS NULL
        ORDER BY start_time
    """)
    rows = cur.fetchall()
    print(f"\n=== Embedding {len(rows)} context_slices ===")
    if len(rows) == 0:
        print("  No slices need embedding.")
        return 0

    ids = [r[0] for r in rows]
    texts = []
    for r in rows:
        summary, topics = r[1], r[2]
        topics_list = json.loads(topics) if topics else []
        combined = f"{summary} | Topics: {', '.join(topics_list)}"
        texts.append(combined)

    embeddings = generate_embeddings(model, texts)
    generated_at = datetime.now(timezone.utc).isoformat()

    count = 0
    for i, (slice_id, text) in enumerate(zip(ids, texts)):
        emb = embeddings[i]
        emb_bytes = emb.astype(np.float32).tobytes()
        cur.execute("""
            UPDATE context_slices
            SET embedding = ?,
                embedding_model = ?,
                embedding_dimension = ?,
                embedding_generated_at = ?,
                embedding_text = ?
            WHERE id = ?
        """, (emb_bytes, MODEL_NAME, DIM, generated_at, text, slice_id))
        count += 1

    conn.commit()
    print(f"  Embedded {count} context_slices")
    return count

def print_stats(conn):
    cur = conn.cursor()

    cur.execute("SELECT COUNT(*) FROM memory_events WHERE embedding IS NOT NULL AND evidence_source = 'imported'")
    events_with_emb = cur.fetchone()[0]

    cur.execute("SELECT COUNT(*) FROM memory_events WHERE evidence_source = 'imported'")
    total_imported = cur.fetchone()[0]

    cur.execute("SELECT COUNT(*) FROM context_slices WHERE embedding IS NOT NULL")
    slices_with_emb = cur.fetchone()[0]

    cur.execute("SELECT COUNT(*) FROM context_slices")
    total_slices = cur.fetchone()[0]

    print("\n=== Embedding Stats ===")
    print(f"Imported events: {events_with_emb}/{total_imported} with 1024-dim Qwen3 embedding")
    print(f"Context slices:  {slices_with_emb}/{total_slices} with 1024-dim Qwen3 embedding")

    return events_with_emb, slices_with_emb

def recall_top_k(model, conn, query, k=10):
    query_emb = model.encode([query], normalize_embeddings=True)[0]

    cur = conn.cursor()

    cur.execute("""
        SELECT id, content, window_title, embedding, evidence_source, timestamp
        FROM memory_events
        WHERE embedding IS NOT NULL
    """)
    event_rows = cur.fetchall()

    cur.execute("""
        SELECT id, summary, embedding, evidence_source, start_time, topics
        FROM context_slices
        WHERE embedding IS NOT NULL
    """)
    slice_rows = cur.fetchall()

    results = []

    for row in event_rows:
        emb = np.frombuffer(row[3], dtype=np.float32)
        sim = cosine_sim(query_emb, emb)
        results.append({
            'source_type': 'memory_event',
            'id': row[0],
            'content': row[1],
            'window_title': row[2],
            'evidence_source': row[4],
            'timestamp': row[5],
            'similarity': float(sim)
        })

    for row in slice_rows:
        emb = np.frombuffer(row[2], dtype=np.float32)
        sim = cosine_sim(query_emb, emb)
        topics = json.loads(row[5]) if row[5] else []
        results.append({
            'source_type': 'context_slice',
            'id': row[0],
            'summary': row[1],
            'topics': topics,
            'evidence_source': row[3],
            'start_time': row[4],
            'similarity': float(sim)
        })

    results.sort(key=lambda x: x['similarity'], reverse=True)
    return results[:k]

def run_recall_tests(model, conn):
    test_queries = [
        "弹窗提醒无效 reason 太薄 无法给执行端有效上下文",
        "Local Agent Box payload_text Relay dispatch 树莓派硬件链路",
        "不要为了方便牺牲体验 不要程序硬编码语义",
        "向量模型 embedding Qwen3 记忆检索",
    ]

    print("\n\n" + "="*80)
    print("RECALL TEST RESULTS")
    print("="*80)

    all_results = {}

    for qi, query in enumerate(test_queries):
        print(f"\n--- Query {qi+1}: {query} ---")
        results = recall_top_k(model, conn, query, k=10)
        all_results[query] = results

        for ri, r in enumerate(results):
            source = r['source_type']
            if source == 'memory_event':
                content = r['content'][:100] + '...' if len(r['content']) > 100 else r['content']
                print(f"  [{ri+1}] [{source}] id={r['id']} sim={r['similarity']:.4f} src={r['evidence_source']}")
                print(f"      content: {content}")
                print(f"      window: {r['window_title']}")
            else:
                topics = r.get('topics', [])
                print(f"  [{ri+1}] [{source}] id={r['id']} sim={r['similarity']:.4f} src={r['evidence_source']}")
                print(f"      summary: {r['summary'][:100]}...")
                print(f"      topics: {topics}")
        print()

    return all_results

def main():
    print("="*80)
    print("Real Memory Embedding Integration")
    print("Model: Qwen/Qwen3-Embedding-0.6B via sentence-transformers Route A")
    print("="*80)

    migrate_db()
    model = load_model()

    conn = sqlite3.connect(DB_PATH)

    events_count = embed_memory_events(model, conn)
    slices_count = embed_context_slices(model, conn)

    print_stats(conn)

    recall_results = run_recall_tests(model, conn)

    conn.close()

    # Write batch report
    import json
    report_path = "D:/C_Projects/Agent/cozmio/verification/embedding_batch_report.md"
    generated_at = datetime.now(timezone.utc).isoformat()

    with open(report_path, 'w', encoding='utf-8') as f:
        f.write("# Real Memory Embedding Integration — Batch Embedding Report\n\n")
        f.write(f"**Generated**: {generated_at}\n\n")
        f.write("## Model\n\n")
        f.write("- **Model**: Qwen/Qwen3-Embedding-0.6B\n")
        f.write("- **Route**: A — sentence-transformers 3.0.1 (local)\n")
        f.write(f"- **model_path**: {MODEL_PATH}\n")
        f.write(f"- **Dimension**: 1024\n")
        f.write(f"- **Pooling**: pooling_mode_lasttoken=True (official)\n\n")
        f.write("## Embedding Coverage\n\n")
        f.write(f"- **Imported events embedded**: {events_count}\n")
        f.write(f"- **Context slices embedded**: {slices_count}\n\n")

        for qi, query in enumerate(test_queries):
            results = recall_results[query]
            f.write(f"## Query {qi+1}: {query}\n\n")
            f.write("| # | source | id | sim | src | text |\n")
            f.write("|---|--------|----|----|----|----|\n")
            for ri, r in enumerate(results):
                source = r['source_type']
                if source == 'memory_event':
                    content = r['content'][:60] + '...' if len(r['content']) > 60 else r['content']
                    f.write(f"| {ri+1} | {source} | {r['id']} | {r['similarity']:.4f} | {r['evidence_source']} | {content} |\n")
                else:
                    summary = r['summary'][:60] + '...' if len(r['summary']) > 60 else r['summary']
                    f.write(f"| {ri+1} | {source} | {r['id']} | {r['similarity']:.4f} | {r['evidence_source']} | {summary} |\n")
            f.write("\n")

    print(f"\nBatch report written to: {report_path}")
    print("Done.")

if __name__ == "__main__":
    main()