let kyberInstance;
let kyberPublicKey;
let kyberPrivateKey;


function arrayBufferToBase64(buffer) {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    bytes.forEach(b => binary += String.fromCharCode(b));
    return window.btoa(binary);
}

// Utility function to convert Base64 string to ArrayBuffer
function base64ToArrayBuffer(base64) {
    const binary = window.atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
    }
    return bytes.buffer;
}

async function generateKyberKeyPair() {
      try {
        kyberInstance = new Kyber1024();
        [kyberPublicKey, kyberPrivateKey] = await kyberInstance.generateKeyPair();
        console.log("Kyber keys inititated"); 
      } catch (err) {
        console.error("Error generating key pair:", err.message);
      }
    }


function hexToArrayBuffer(hex) {
      const length = hex.length / 2;
      const buffer = new ArrayBuffer(length);
      const view = new Uint8Array(buffer);
      for (let i = 0; i < length; i++) {
        view[i] = parseInt(hex.substr(i * 2, 2), 16);
      }
      return buffer;
    }

    // Helper function to convert ArrayBuffer to Base64 string
    function arrayBufferToBase64(buffer) {
      let binary = '';
      const bytes = new Uint8Array(buffer);
      for (let i = 0; i < bytes.byteLength; i++) {
        binary += String.fromCharCode(bytes[i]);
      }
      return window.btoa(binary);
    }

    // AES encryption function using a 256-bit key (no IV)
async function encryptMessage(message, keyHex) {
    // Convert hex key to ArrayBuffer
    const keyBuffer = hexToArrayBuffer(keyHex);

    // Encode the message to ArrayBuffer
    const encoder = new TextEncoder();
    const messageBuffer = encoder.encode(message);

    // Generate a random initialization vector (IV)
    const iv = window.crypto.getRandomValues(new Uint8Array(12)); // 12-byte IV for AES-GCM

    // Import the key for AES-GCM
    const key = await window.crypto.subtle.importKey(
        "raw",
        keyBuffer,
        { name: "AES-GCM", length: 256 }, // AES-GCM with 256-bit key
        false,
        ["encrypt"]
    );

    // Perform AES encryption
    const encryptedBuffer = await window.crypto.subtle.encrypt(
        { name: "AES-GCM", iv: iv },
        key,
        messageBuffer
    );
  
    // Combine the IV and encrypted message
    const encryptedArray = new Uint8Array(encryptedBuffer);
    const ivAndEncrypted = new Uint8Array(iv.byteLength + encryptedArray.byteLength);
    ivAndEncrypted.set(iv, 0);
    ivAndEncrypted.set(encryptedArray, iv.byteLength);

    // Convert combined ArrayBuffer to hex string
    return arrayBufferToHex(ivAndEncrypted);
}

// Helper function to convert ArrayBuffer to hex string
function arrayBufferToHex(buffer) {
    return Array.from(new Uint8Array(buffer))
        .map(byte => byte.toString(16).padStart(2, '0'))
        .join('');
}


async function decryptMessage(encryptedMessageHex, keyHex) {
    // Convert hex key to ArrayBuffer
    const keyBuffer = hexToArrayBuffer(keyHex);

    // Convert the encrypted message hex to ArrayBuffer
    const encryptedBuffer = hexToArrayBuffer(encryptedMessageHex);

    // Extract the first 12 bytes as the IV
    const iv = encryptedBuffer.slice(0, 12);

    // The remaining bytes are the actual encrypted message
    const cipherTextBuffer = encryptedBuffer.slice(12);

    // Import the key for AES-GCM
    const key = await window.crypto.subtle.importKey(
        "raw",
        keyBuffer,
        { name: "AES-GCM", length: 256 },
        false,
        ["decrypt"]
    );

    // Perform AES decryption
    const decryptedBuffer = await window.crypto.subtle.decrypt(
        { name: "AES-GCM", iv: iv },
        key,
        cipherTextBuffer
    );

    // Decode the decrypted ArrayBuffer to a string
    const decoder = new TextDecoder();
    console.log("Decoded message", decoder.decode(decryptedBuffer));
    return decoder.decode(decryptedBuffer);
}
