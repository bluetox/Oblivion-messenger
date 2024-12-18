async function hashPublicKey() {
    const hashBuffer = await crypto.subtle.digest('SHA-256', key);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const hashHex = hashArray.map(byte => byte.toString(16).padStart(2, '0')).join('');
    return hashHex;
  }