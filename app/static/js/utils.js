function getCookie() {

    const name = "session_id";
    const value = `; ${document.cookie}`;
    const parts = value.split(`; ${name}=`);

    if (parts.length === 2) {
        return parts.pop().split(';').shift();
    }

    return null;
}

function addCircle(destId) {

    const button = document.querySelector(`.sidebar-button[data-dest-id="${destId}"]`);

    if (button) {

        const greenCircle = document.createElement('div');
        greenCircle.classList.add('green-circle');
        
        button.appendChild(greenCircle);
    }
}

function removeAllGreenCircles() {

    const buttons = document.querySelectorAll('.sidebar-button');

    buttons.forEach(button => {
        const greenCircle = button.querySelector('.green-circle');
        
        if (greenCircle) {
            greenCircle.remove();
        }
    });
}

async function establishWebSocketConnection() {
    await generateKyberKeyPair();
    
            
    socket = io.connect({
        transports: ['websocket'],
        origin: '*',
        withCredentials: true,
        secure: true
    });

    socket.on('connect', () => {
        console.log("SocketIO connexion with the server established");
    });

    setInterval(() => {

        socket.emit('get_status',{ allDestIds });
    }, 2000);

    socket.on('onlines', (data) => {
        removeAllGreenCircles();
        onlineIds = data.online_user_ids;
        for (let i = 0; i < onlineIds.length; i++) {
            addCircle(onlineIds[i]);
        }

    });

    socket.on('append_KyberKey', async (data) => {
        if (!allDestIds.includes(data.source_id)) {
            try {
                let invites = JSON.parse(localStorage.getItem('invites')) || {};
                invites[data.source_id] = Date.now();
                localStorage.setItem('invites', JSON.stringify(invites));
            } catch (error) {
                console.error("Error accessing or updating invites in localStorage", error);
            }
            return
        }
        try {
    
            const [cypherText, sharedSecretTemp] = await kyberInstance.encap(new Uint8Array(data.public_key));
            sharedSecret[data.source_id] = sharedSecretTemp;
    
            const cypherTextHex = convertToHex(cypherText);
            let sharedSecrets = JSON.parse(localStorage.getItem('shared_secrets')) || {};
            sharedSecrets[data.source_id] = await encryptData(sharedSecret[data.source_id], password);
            localStorage.setItem('shared_secrets', JSON.stringify(sharedSecrets));
            socket.emit('append_cypher', { cypherText: cypherTextHex ,dest_id : data.source_id});
        } catch (err) {
            console.error("Error during key exchange:", err.message);
        }
    });
    
    socket.on('append_cypher', async (data) => {
        if (!allDestIds.includes(data.from_user_id)) {
            try {
                let invites = JSON.parse(localStorage.getItem('invites')) || {};
                invites[data.from_user_id] = Date.now();
                localStorage.setItem('invites', JSON.stringify(invites));
            } catch (error) {
                console.error("Error accessing or updating invites in localStorage", error);
            }
            return
        }
        try {
    
            const cypherTextBinary = hexToUint8Array(data.cypherText);
            sharedSecret[data.from_user_id] = await kyberInstance.decap(cypherTextBinary, PrivateKeyList[data.from_user_id]);

            let sharedSecrets = JSON.parse(localStorage.getItem('shared_secrets')) || {};

            sharedSecrets[data.from_user_id] = await encryptData(sharedSecret[data.from_user_id], password);

            localStorage.setItem('shared_secrets', JSON.stringify(sharedSecrets));
        } catch (err) {
            console.error("Error during decapsulation:", err.message);
        }
    });
    
    socket.on('receive_message', async function(data) {
        if (!allDestIds.includes(data.from_user_id)) {
            return
        }
        const message = hexToArrayBuffer(data.message);
        try {
            const decryptedMessageArray = await decryptMessage(message, sharedSecret[data.from_user_id], "message");
            const decryptedMessage = uint8ArrayToString(decryptedMessageArray);
            let encodedMessage = new TextEncoder().encode(decryptedMessage);
            if (!validateDilithiumSignature(encodedMessage, data.signature, data.from_user_id)) {
                console.warn("Received a message with erroneous signing from user: ", data.from_user_id);
                return;
            };
            addMessageToHistory(decryptedMessage, "user", currentChatNum, data.from_user_id);
            
        } catch (error) {
            addMessageToHistory(`Failed to decrypt message from ${data.from_user_id}.`, "error");
        }
    });

    socket.on('receivedFile', async function(response) {
        const base64FileData = response.file;
        const fileName = response.fileName;
        const binaryData = atob(base64FileData);
        
        const decryptedFileData = new Uint8Array(binaryData.split('').map(char => char.charCodeAt(0)));
        const decryptedContent = await decryptFile(decryptedFileData, sharedSecret[currentChatDestUserId]);
        const decryptedBlob = new Blob([decryptedContent], { type: 'application/octet-stream' });
        
        const link = document.createElement('a');
        link.href = URL.createObjectURL(decryptedBlob);
        link.download = `${fileName}`; 
        link.click();
    });

    socket.on('append_dilithium_key', function(data) {
        const key = data.key;
        const sourceId = data.source_id;
        dilithium_keys[sourceId] = key;

    });
}

async function fetchUserData() {
    try {
        const response = await fetch(`/api/get-user-data`);
        if (response.status != 200) {
            await createNewSession();
            return;
        }
        const data = await response.json();
        sessionUserId = data.user_id;

        document.getElementById("userId").innerHTML = `<h2>Your User ID:</h2><pre>${sessionUserId}</pre>`;
        await establishWebSocketConnection();
    } catch (error) {
        console.error("Error fetching user data:", error);
    }
}

async function createNewSession() {
    try {

        const response = await fetch('api/create_cookies');
        const data = await response.json();
        
        document.cookie = `session_id=${data.session_cookie}; path=/; expires=Fri, 31 Dec 2024 23:59:59 GMT`;
        console.log("cookies set");
        sessionUserId = data.user_id;
        document.getElementById("userId").innerHTML = `<h2>Your User ID:</h2><pre>${sessionUserId}</pre>`;
        await establishWebSocketConnection();
    } catch (error) {
        console.error("Error in creating new session:", error);
    }
}

async function setup() {

    const sessionId = getCookie("session_id");

    if (sessionId) {
        await fetchUserData(sessionId);
    } else {
        await createNewSession();
    }
}


function clearChat() {

    const messagesContainer = document.querySelector('.messages');
    while (messagesContainer.firstChild) {

        messagesContainer.removeChild(messagesContainer.firstChild);
    }
}

async function removeChat() {

    const button = document.querySelector(`.sidebar-button[data-chatid="${currentChatNum}"]`);
    findUserIdById(currentChatNum).then(function (userId) {
        let sharedSecrets = JSON.parse(localStorage.getItem('shared_secrets')) || {};
        delete sharedSecrets[userId];
        localStorage.setItem('shared_secrets', JSON.stringify(sharedSecrets));
        delete sharedSecret[userId];
    });
    
    await removeChatFromDb();
    

    clearChat();
    if (button) {
        button.remove();
    } else {
        console.log("Button not found");
    }
}
function getSharedSecrets() {

    const sharedSecrets = JSON.parse(localStorage.getItem('shared_secrets')) || {};
    
    return sharedSecrets
}

async function decryptSharedSecrets() {

    const shared_secrets = getSharedSecrets();
    for (const user_id in shared_secrets) {
        const decryptedSecret = await decryptMessage(shared_secrets[user_id], password, "file");        
        sharedSecret[user_id] =  decryptedSecret;
    }
}

async function getOfflineMessages() {
    try {

        const messages = await new Promise((resolve) => {
            socket.emit('get_messages', resolve);
        });
        for (const message of messages) {
            const decryptedMessage = await decryptOfflineMessages(
                message.message,
                sharedSecret[message.from_user_id]
            );
            findIdByUserId(message.from_user_id).then(id => {
                addMessageToHistory(decryptedMessage, "user", id, message.from_user_id);
            });
            
        }
    } catch (error) {
        console.error('Error retrieving messages:', error);
    }
}

async function decryptOfflineMessages(hexMessage, sharedSecret) {
    try {
        const encryptedData = Uint8Array.from(
            hexMessage.match(/.{1,2}/g).map(byte => parseInt(byte, 16))
        );

        const nonce = encryptedData.slice(0, 12);
        const ciphertext = encryptedData.slice(12);
        const key = await crypto.subtle.importKey(
            "raw",
            sharedSecret,
            { name: "AES-GCM" },
            false,
            ["decrypt"]
        );

        const decryptedData = await crypto.subtle.decrypt(
            { name: "AES-GCM", iv: nonce },
            key,
            ciphertext
        );

        const decoder = new TextDecoder();
        return decoder.decode(decryptedData);
    } catch (error) {
        console.error("Failed to decrypt message:", error);
        return null;
    }
}

function stringToUint8Array(plaintext) {
    const encoder = new TextEncoder();
    return encoder.encode(plaintext);
}

function uint8ArrayToString(array) {
    const decoder = new TextDecoder();
    return decoder.decode(array);
}

function convertToHex(key) {

    let hex = '';

    for (let i = 0; i < key.length; i++) {
      hex += key[i].toString(16).padStart(2, '0');
    }

    return hex;
}

function hexToUint8Array(hex) {

  const bytes = new Uint8Array(hex.length / 2);

  for (let i = 0; i < hex.length; i += 2) {
      bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
  }

  return bytes;
}

function loadInvites() {
    document.querySelectorAll('.sidebar-button').forEach((element) => {
        if (areChatsDisplayed) {
            element.style.display = 'none';
        } else {
            element.style.display = 'block';
        }
    });

    document.querySelectorAll('.invite-button').forEach((element) => {
        if (areChatsDisplayed) {
            element.style.display = 'block';
        } else {
            element.style.display = 'none';
        }
    });

    if (areChatsDisplayed) {
        let invites = JSON.parse(localStorage.getItem('invites')) || {};

        const sidebar = document.querySelector('.sidebar');
        if (!sidebar) {
            console.error('Sidebar container not found!');
            return;
        }

        sidebar.querySelectorAll('.invite-button').forEach(button => button.remove());

        for (const invite in invites) {
            const newButton = document.createElement('button');
            newButton.classList.add('invite-button');
            newButton.textContent = `Invite from ${invite}`;
            newButton.dataset.userId = invite;
            newButton.onclick = async () => {
                document.getElementById("user_id").value = invite;
                openModal();
            };
            sidebar.appendChild(newButton);
        }
    }

    areChatsDisplayed = !areChatsDisplayed;
}
