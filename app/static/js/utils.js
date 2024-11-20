let sharedSecret = {};
// Function to return the session cookie if it exists
function getCookie() {
    const name = "session_id"; // Cookie name to get
    const value = `; ${document.cookie}`; // Store the whole cookie container
    const parts = value.split(`; ${name}=`); // Store the different cookies in an array
    if (parts.length === 2) {
        return parts.pop().split(';').shift();  // Return the session_id
    }
    return null;  // Return null if the cookie does not exist
}


// Function to establish a WebSocket connection
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
        onlineIds = data.online_user_ids;
        console.log("Online right now: ", onlineIds);
    });

    

    // Wait for the server's response with the other party's public key
    socket.on('append_KyberKey', async (data) => {
        try {
            const kyber = new Kyber1024();
    
            // Encapsulate the shared secret using the received public key
            const [cypherText, sharedSecretTemp] = await kyber.encap(new Uint8Array(data.public_key));
            sharedSecret[data.source_id] = sharedSecretTemp; // Store the shared secret for later use
    
            // Convert the cypherText to hex for sending to the server
            const cypherTextHex = convertToHex(cypherText);
            console.log("Shared Secret established");
            
            // Emit the cyphertext to the server as a hex string
            socket.emit('append_cypher', { cypherText: cypherTextHex ,dest_id : data.source_id});
        } catch (err) {
            console.error("Error during key exchange:", err.message);
        }
    });
    
    // Event listener for receiving cyphertext from the other party
    socket.on('append_cypher', async (data) => {
        try {
            console.log("Cyphertext received");
    
            // Convert the received cypherText from hex back to binary format
            const cypherTextBinary = hexToUint8Array(data.cypherText);
    
            // Decapsulate to derive the shared secret
            const kyber = new Kyber1024();  // Ensure `kyber` is defined here as well
            sharedSecret[data.from_user_id] = await kyber.decap(cypherTextBinary, kyberPrivateKey);
    
            console.log("Shared Secret established");
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

// Function to fetch user data from the server
async function fetchUserData(sessionId) {
    try {
        const response = await fetch(`/api/get-user-data?session_id=${sessionId}`);
        const data = await response.json();
        userId = data.user_id;
        document.getElementById("userId").innerHTML = `<h2>Your User ID:</h2><pre>${userId}</pre>`;
        console.log("Successfully got user data and RSA key");
        await establishWebSocketConnection(); // Establish WebSocket connection after fetching user data
    } catch (error) {
        console.error("Error fetching user data:", error);
    }
}

// Function to create a new session if session_id does not exist
async function createNewSession() {
    try {
        const response = await fetch('api/create_cookies');
        const data = await response.json();
        document.cookie = `session_id=${data.session_cookie}; path=/; expires=Fri, 31 Dec 2024 23:59:59 GMT`;
        userId = data.user_id;
        document.getElementById("userId").innerHTML = `<h2>Your User ID:</h2><pre>${userId}</pre>`;
        console.log("Successfully got cookie, RSA key, and user ID");
        await establishWebSocketConnection(); // Establish WebSocket connection after session creation
    } catch (error) {
        console.error("Error in creating new session:", error);
    }
}

// Main setup function
async function setup() {
    const sessionId = getCookie("session_id");

    if (sessionId) {
        await fetchUserData(sessionId); // Fetch user data if session exists
    } else {
        await createNewSession(); // Create new session if no session_id
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