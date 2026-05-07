// Verification script for Cozmio Toast action buttons
// Run with: node verify_protocol.js

const { invoke } = require('@tauri-apps/api/core');

async function runVerification() {
  console.log('=== Cozmio Toast Action Verification ===\n');

  // Step 1: Reset verification state
  console.log('Step 1: Reset verification state');
  try {
    const resetResult = await invoke('reset_verification');
    console.log('  Reset:', resetResult);
  } catch (e) {
    console.log('  Reset error:', e);
  }

  // Step 2: Send a verification Toast
  console.log('\nStep 2: Send verification Toast');
  try {
    const toastResult = await invoke('send_verification_toast', { trace_id: 'test-verify-123' });
    console.log('  Toast sent:', toastResult);
  } catch (e) {
    console.log('  Toast error:', e);
  }

  // Step 3: Get verification result after a delay
  console.log('\nStep 3: Get verification result');
  await new Promise(r => setTimeout(r, 2000));
  try {
    const result = await invoke('get_verification_result');
    console.log('  Verification result:', JSON.stringify(result, null, 2));
  } catch (e) {
    console.log('  Get result error:', e);
  }

  console.log('\n=== Manual verification needed ===');
  console.log('1. Check if Windows Toast appeared with two action buttons');
  console.log('2. Click "确认" button and check if app receives action=confirm');
  console.log('3. Click "取消" button and check if app receives action=cancel');
  console.log('4. Verify main window does NOT show when clicking action buttons');
}

runVerification().catch(console.error);
