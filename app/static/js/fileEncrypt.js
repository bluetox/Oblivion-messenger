async function decryptFile(encryptedData, keybuffer) {
    const iv = encryptedData.slice(0, 12);
    const encryptedContent = encryptedData.slice(12);
    
    try {
        const importedKey = await crypto.subtle.importKey(
            "raw", keybuffer, { name: "AES-GCM" }, false, ["decrypt"]
        );
  
        const decryptedContent = await crypto.subtle.decrypt(
            { name: "AES-GCM", iv: iv },
            importedKey,
            encryptedContent
        );
  
        return new Uint8Array(decryptedContent);
    } catch (err) {
        console.error("Error during decryption: ", err);
        throw new Error("Decryption failed");
    }
}

async function encryptFile(file, keybuffer) {

    const iv = crypto.getRandomValues(new Uint8Array(12));
    const tagLength = 128;
    const fileArrayBuffer = await file.arrayBuffer();
    
    try {
        const importedKey = await crypto.subtle.importKey(
            "raw", keybuffer, { name: "AES-GCM" }, false, ["encrypt"]
        );
        
        const encryptedContent = await crypto.subtle.encrypt(
            { name: "AES-GCM", iv: iv, tagLength: tagLength },
            importedKey,
            fileArrayBuffer
        );
        
        const encryptedDataWithTag = new Uint8Array(encryptedContent);
        const result = new Uint8Array(iv.length + encryptedDataWithTag.length);
        result.set(iv, 0);
        result.set(encryptedDataWithTag, iv.length);

        return result;
    } catch (err) {
        console.error("Error during encryption: ", err);
        throw new Error("Encryption failed");
    }
}

async function sendFile() {
    const fileInput = document.getElementById('fileInput');
    if (fileInput.files.length === 0) {
        alert('Please select a file to encrypt.');
        return;
    }

    const file = fileInput.files[0];
    const fileName = file.name;
    const keyHex = sharedSecret[currentChatDestUserId];
    const encryptedData = await encryptFile(file, keyHex); 

    const encryptedBlob = new Blob([encryptedData], { type: 'application/octet-stream' });

    message = {
        fileName: fileName,
        file: encryptedBlob,
        dest: currentChatDestUserId
    }
    socket.emit('encryptedFile', message);
    delete fileInput.files[0];
}
