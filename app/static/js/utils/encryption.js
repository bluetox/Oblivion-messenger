export async function generateKyberKeyPair() {
      try {
        [kyberPublicKey, kyberPrivateKey] = await kyberInstance.generateKeyPair();
        console.log("Kyber keys inititated"); 
      } catch (err) {
        console.error("Error generating key pair:", err.message);
      }
}

export async function deriveKey(password, salt) {
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

export async function encryptMessage(message, keyBuffer) {

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
        message
    );
  
    const encryptedArray = new Uint8Array(encryptedBuffer);
    const ivAndEncrypted = new Uint8Array(iv.byteLength + encryptedArray.byteLength);

    ivAndEncrypted.set(iv, 0);
    ivAndEncrypted.set(encryptedArray, iv.byteLength);
    return ivAndEncrypted;
}

export async function decryptMessage(encryptedBuffer, keyBuffer, type) {


    if (type === "message") {
        const iv = encryptedBuffer.slice(0, 12);
        const cipherTextBuffer = encryptedBuffer.slice(12);
        const key = await window.crypto.subtle.importKey(
            "raw",
            keyBuffer,
            { name: "AES-GCM", length: 256 },
            false,
            ["decrypt"]
        );

        try {
            const decryptedBuffer = await window.crypto.subtle.decrypt(
                { name: "AES-GCM", iv: iv },
                key,
                cipherTextBuffer
            );
            return decryptedBuffer;
        } catch (error) {
            console.error("Decryption failed:", error);
            throw new Error("Decryption failed: " + error.message);
        }
    }
    if (type === "file") {
        try {

            const dataBuffer = new Uint8Array(atob(encryptedBuffer).split("").map(c => c.charCodeAt(0)));
            const salt = dataBuffer.slice(0, 16);
            const nonce = dataBuffer.slice(16, 28);
            const encryptedData = dataBuffer.slice(28);
            const aesKey = await deriveKey(keyBuffer, salt);

            const decryptedData = await window.crypto.subtle.decrypt(
                { name: "AES-GCM", iv: nonce },
                aesKey,
                encryptedData
            );
    
            return decryptedData;

        } catch (error) {
            console.error("Error during decryption:", error);
        }
    }
}

export async function encryptData(data, password) {

    const salt = crypto.getRandomValues(new Uint8Array(16));
    const nonce = crypto.getRandomValues(new Uint8Array(12));
    const aesKey = await deriveKey(password, salt);
    const encryptedData = await window.crypto.subtle.encrypt(
        { name: "AES-GCM", iv: nonce },
        aesKey,
        data
    );
    const combinedData = new Uint8Array(salt.byteLength + nonce.byteLength + encryptedData.byteLength);
    combinedData.set(new Uint8Array(salt.buffer), 0);
    combinedData.set(new Uint8Array(nonce.buffer), salt.byteLength);
    combinedData.set(new Uint8Array(encryptedData), salt.byteLength + nonce.byteLength);
    return btoa(String.fromCharCode(...combinedData));
}

export async function hashData(rawData) {
    const data = new TextEncoder().encode(rawData)
    const hashBuffer = await crypto.subtle.digest('SHA-256', data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const hashHex = hashArray.map(byte => byte.toString(16).padStart(2, '0')).join('');
    return hashHex;
}

async function generateAesGcmKey() {
    try {
        const key = await crypto.subtle.generateKey(
            {
                name: "AES-GCM",
                length: 256,
            },
            true,
            ["encrypt", "decrypt"]
        );

        console.log("AES-GCM Key successfully generated!");
        const rawKey = await crypto.subtle.exportKey("raw", key);
        const base64Key = btoa(String.fromCharCode(...new Uint8Array(rawKey)));
        return base64Key;

    } catch (error) {
        console.error("Error generating AES-GCM key:", error);
    }
}
export async function generateDecryptionKey(destUserId) {
    const base64Key = await generateAesGcmKey();
    const data = {key: base64Key, user_id: destUserId}
    let keys = JSON.parse(localStorage.getItem('keys')) || {};
    keys[destUserId] = base64Key;
    localStorage.setItem('keys', JSON.stringify(keys));
    fetch('/api/set_decryption_key', {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        credentials: "include",

        body: JSON.stringify(data)
    })
    .then(response => {
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        return response.json();
    })
    .then(result => {
        console.log("Response from server:", result);
    })
}
export function getDecryptionKey(destUserId) {
    const url = `/api/get_decryption_key?user_id=${encodeURIComponent(destUserId)}`;

    return fetch(url, {
        method: "GET",
        headers: {
            "Content-Type": "application/json"
        },
        credentials: "include",
    })
        .then(response => {
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            return response.json();
        })
        .then(data => {
            return data.key;
        })
        .catch(error => {
            console.error("Error:", error);
            throw error;
        });
}






export default {
    hashData,
    encryptData,
    decryptMessage,
    encryptMessage,
    deriveKey,
    generateKyberKeyPair
};
