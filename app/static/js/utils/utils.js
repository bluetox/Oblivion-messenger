import {hexToArrayBuffer} from '/static/js/utils/convertions.js'
import {addMessageToHistory} from '/static/js/utils/uiChatInteract.js'
import {encryptData, decryptMessage, generateKyberKeyPair} from '/static/js/utils/encryption.js';
import {findUserIdById, removeChatFromDb} from '/static/js/utils/database.js';
import {validateDilithiumSignature} from '/static/js/utils/signing.js';
import {decryptFile} from '/static/js/utils/fileEncrypt.js'

export function getCookie() {
    const name = "session_id";
    const value = `; ${document.cookie}`;
    const parts = value.split(`; ${name}=`);

    if (parts.length === 2) {
        return parts.pop().split(';').shift();
    }

    return null;
}
export function setCookie(name, value, days) {
    const date = new Date();
    date.setTime(date.getTime() + (days * 24 * 60 * 60 * 1000));
    const expires = "expires=" + date.toUTCString();
    document.cookie = `${name}=${value}; ${expires}; path=/; Secure`;
}
export function addUsersStatus(onlineIds) {
    for (let i = 0; i < onlineIds.length; i++) {
        const button = document.querySelector(`.sidebar-button[data-dest-id="${onlineIds[i]}"]`);

        if (button) {
            const greenCircle = document.createElement('div');
            greenCircle.classList.add('green-circle');

            button.appendChild(greenCircle);
        }
    }
}
export function removeAllGreenCircles() {

    const buttons = document.querySelectorAll('.sidebar-button');

    buttons.forEach(button => {
        const greenCircle = button.querySelector('.green-circle');
        
        if (greenCircle) {
            greenCircle.remove();
        }
    });
}
export async function establishWebSocketConnection() {
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
        addUsersStatus(onlineIds);

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
            console.log(error);
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
export async function fetchUserData() {
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
export async function createNewSession() {
    try {

        const response = await fetch('api/create_cookies');
        const data = await response.json();
        
        setCookie('session_id',data.session_cookie,7);
        sessionUserId = data.user_id;
        document.getElementById("userId").innerHTML = `<h2>Your User ID:</h2><pre>${sessionUserId}</pre>`;
        await establishWebSocketConnection();
    } catch (error) {
        console.error("Error in creating new session:", error);
    }
}
export async function setup() {

    const sessionId = getCookie("session_id");

    if (sessionId) {
        await fetchUserData(sessionId);
    } else {
        await createNewSession();
    }
}
export function clearChat() {

    const messagesContainer = document.querySelector('.messages');
    while (messagesContainer.firstChild) {

        messagesContainer.removeChild(messagesContainer.firstChild);
    }
}
export async function removeChat() {

    const button = document.querySelector(`.sidebar-button[data-chatid="${currentChatNum}"]`);
    findUserIdById(currentChatNum).then(function (userId) {
        let sharedSecrets = JSON.parse(localStorage.getItem('shared_secrets')) || {};
        delete sharedSecrets[userId];
        localStorage.setItem('shared_secrets', JSON.stringify(sharedSecrets));
    });
    
    await removeChatFromDb();
    
    clearChat();
    if (button) {
        button.remove();
    } else {
        console.log("Button not found");
    }
}
export function getSharedSecrets() {

    const sharedSecrets = JSON.parse(localStorage.getItem('shared_secrets')) || {};
    
    return sharedSecrets
}
export async function decryptSharedSecrets() {

    const shared_secrets = getSharedSecrets();
    for (const user_id in shared_secrets) {
        const decryptedSecret = await decryptMessage(shared_secrets[user_id], password, "file");        
        sharedSecret[user_id] =  decryptedSecret;
    }
}
export async function getOfflineMessages() {
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
export async function decryptOfflineMessages(hexMessage, sharedSecret) {
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
export function stringToUint8Array(plaintext) {
    const encoder = new TextEncoder();
    return encoder.encode(plaintext);
}
export function uint8ArrayToString(array) {
    const decoder = new TextDecoder();
    return decoder.decode(array);
}
export function convertToHex(key) {

    let hex = '';

    for (let i = 0; i < key.length; i++) {
      hex += key[i].toString(16).padStart(2, '0');
    }

    return hex;
}
export function hexToUint8Array(hex) {

  const bytes = new Uint8Array(hex.length / 2);

  for (let i = 0; i < hex.length; i += 2) {
      bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
  }

  return bytes;
}
export function loadInvites() {
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
