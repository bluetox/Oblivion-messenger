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
    socket = io.connect(ngrokUrl, {
        transports: ['websocket'],
        withCredentials: true,
        secure: true
    });

    socket.on('connect', () => {
        console.log("SocketIO connexion with the server established");
    });

    setInterval(() => {

        socket.emit('heartbeat');
        socket.emit('get_status',{ allChatIds });
    }, 2000);

    socket.on('onlines', (data) => {
        removeAllGreenCircles();
        onlineIds = data.online_user_ids;
        for (let i = 0; i < onlineIds.length; i++) {
            console.log("trying to add a circle to chat: ", onlineIds[i]);
            addCircle(onlineIds[i]);
        }

    });

    socket.on('append_KyberKey', async (data) => {
        try {
    
            const [cypherText, sharedSecretTemp] = await kyberInstance.encap(new Uint8Array(data.public_key));
            sharedSecret[data.source_id] = sharedSecretTemp;
    
            const cypherTextHex = convertToHex(cypherText);
            console.log("Shared Secret established with: ",data.source_id);
            
            socket.emit('append_cypher', { cypherText: cypherTextHex ,dest_id : data.source_id});
        } catch (err) {
            console.error("Error during key exchange:", err.message);
        }
    });
    
    socket.on('append_cypher', async (data) => {
        try {
    
            const cypherTextBinary = hexToUint8Array(data.cypherText);
            sharedSecret[data.from_user_id] = await kyberInstance.decap(cypherTextBinary, PrivateKeyList[data.from_user_id]);
    
        } catch (err) {
            console.error("Error during decapsulation:", err.message);
        }
    });
    
    socket.on('receive_message', async function(data) {
        console.log('Message received:', data.from_user_id);
        try {
            const decryptedMessage = await decryptMessage(data.message ,convertToHex(sharedSecret[data.from_user_id]));
            addMessageToHistory(decryptedMessage, "user", CurrentChatIndex, data.from_user_id);
        } catch (error) {
            addMessageToHistory(`Failed to decrypt message from ${data.from_user_id}.`, "error");
        }
    });
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

async function fetchUserData(sessionId) {
    try {
        const response = await fetch(`/api/get-user-data?session_id=${sessionId}`);
        if (response.status != 200) {
            await createNewSession();
            return;
        }
        const data = await response.json();
        userId = data.user_id;

        document.getElementById("userId").innerHTML = `<h2>Your User ID:</h2><pre>${userId}</pre>`;
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
        userId = data.user_id;
        document.getElementById("userId").innerHTML = `<h2>Your User ID:</h2><pre>${userId}</pre>`;

        console.log("New session established succesfully with user id: ", userId);
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

function removeChat() {

    const button = document.querySelector(`.sidebar-button[data-chatid="${CurrentChatIndex}"]`);

    clearChat();
    if (button) {

        button.remove();
    } else {
        console.log("Button not found");
    }
}