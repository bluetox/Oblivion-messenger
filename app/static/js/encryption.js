async function generateKyberKeyPair() {
      try {
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

async function encryptMessage(message, keyHex) {

    const keyBuffer = hexToArrayBuffer(keyHex);
    const encoder = new TextEncoder();
    const messageBuffer = encoder.encode(message);
    const iv = window.crypto.getRandomValues(new Uint8Array(12));
    const key = await window.crypto.subtle.importKey(
        "raw",
        keyBuffer,
        { name: "AES-GCM", length: 256 },
        false,
        ["encrypt"]
    );

    const encryptedBuffer = await window.crypto.subtle.encrypt(
        { name: "AES-GCM", iv: iv },
        key,
        messageBuffer
    );
  
    const encryptedArray = new Uint8Array(encryptedBuffer);
    const ivAndEncrypted = new Uint8Array(iv.byteLength + encryptedArray.byteLength);

    ivAndEncrypted.set(iv, 0);
    ivAndEncrypted.set(encryptedArray, iv.byteLength);

    return arrayBufferToHex(ivAndEncrypted);
}

function arrayBufferToHex(buffer) {
    return Array.from(new Uint8Array(buffer))
        .map(byte => byte.toString(16).padStart(2, '0'))
        .join('');
}


async function decryptMessage(encryptedMessageHex, keyHex) {

    const keyBuffer = hexToArrayBuffer(keyHex);
    const encryptedBuffer = hexToArrayBuffer(encryptedMessageHex);
    const iv = encryptedBuffer.slice(0, 12);
    const cipherTextBuffer = encryptedBuffer.slice(12);
    const key = await window.crypto.subtle.importKey(
        "raw",
        keyBuffer,
        { name: "AES-GCM", length: 256 },
        false,
        ["decrypt"]
    );

    const decryptedBuffer = await window.crypto.subtle.decrypt(
        { name: "AES-GCM", iv: iv },
        key,
        cipherTextBuffer
    );

    const decoder = new TextDecoder();

    return decoder.decode(decryptedBuffer);
}

async function deriveKey(password, salt) {
    const keyMaterial = await window.crypto.subtle.importKey(
        "raw",
        new TextEncoder().encode(password),
        "PBKDF2",
        false,
        ["deriveKey"]
    );
    return window.crypto.subtle.deriveKey(
        {
            name: "PBKDF2",
            salt: salt,
            iterations: 100000,
            hash: "SHA-256"
        },
        keyMaterial,
        { name: "AES-GCM", length: 256 },
        false,
        ["encrypt", "decrypt"]
    );
}

async function encryptData(data, password) {
    const salt = crypto.getRandomValues(new Uint8Array(16));
    const nonce = crypto.getRandomValues(new Uint8Array(12));
    const aesKey = await deriveKey(password, salt);
    const encryptedData = await window.crypto.subtle.encrypt(
        { name: "AES-GCM", iv: nonce },
        aesKey,
        new TextEncoder().encode(data)
    );
    const combinedData = new Uint8Array(salt.byteLength + nonce.byteLength + encryptedData.byteLength);
    combinedData.set(new Uint8Array(salt.buffer), 0);
    combinedData.set(new Uint8Array(nonce.buffer), salt.byteLength);
    combinedData.set(new Uint8Array(encryptedData), salt.byteLength + nonce.byteLength);
    return btoa(String.fromCharCode(...combinedData));
}

async function decryptData(combinedData, password) {
    try {

        const dataBuffer = new Uint8Array(atob(combinedData).split("").map(c => c.charCodeAt(0)));
        const salt = dataBuffer.slice(0, 16);
        const nonce = dataBuffer.slice(16, 28);
        const encryptedData = dataBuffer.slice(28);
        const aesKey = await deriveKey(password, salt);
        const decryptedData = await window.crypto.subtle.decrypt(
            { name: "AES-GCM", iv: nonce },
            aesKey,
            encryptedData
        );

        return new TextDecoder().decode(decryptedData);
    } catch (error) {
        console.error("Error during decryption:", error);

        if (error.name === "OperationError") {
            throw new Error("Decryption failed. Check the password or data integrity.");
        } else if (error.name === "InvalidAccessError") {
            throw new Error("Invalid key or corrupted data.");
        } else {
            throw new Error("An unexpected error occurred during decryption.");
        }
    }
}
