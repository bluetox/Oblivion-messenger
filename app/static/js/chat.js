
            async function addChat() {

                const name = document.getElementById("chat_name").value;
                const destId = document.getElementById("user_id").value;
                const newButton = document.createElement('button');
                const chat_data = { name: name, user_id: destId, timestamp: Date.now() }
                allDestIds.push(destId);

                newButton.classList.add('sidebar-button');

                const chatNum = await saveChat(chat_data);                             
                newButton.textContent = name;
                newButton.dataset.destId = destId;
                newButton.dataset.chatid = chatNum;
                newButton.onclick = async () => {

                    currentChatDestUserId = destId;
                    currentChatNum = chatNum;
                    if (!sharedSecret[destId]) {
                        [publicKey, privateKey] = await kyberInstance.generateKeyPair();
                        PrivateKeyList[destId] = privateKey;
                        socket.emit('append_KyberKey', { public_key: Array.from(publicKey), target_user_id: currentChatDestUserId });
                        
                    }
                    socket.emit('dilithium_key', {key: dilithiumPublicKey.toHex(), 'dest_id' : currentChatDestUserId});
                    clearChat();
                    openChatContainer();
                    loadMessages(chatNum);
                };    
                const sidebar = document.querySelector('.sidebar');
                if (!areChatsDisplayed) {
                    newButton.style.display = 'none';
                }
                sidebar.appendChild(newButton);
                document.getElementById("chat_name").value = '';
                document.getElementById("user_id").value = '';
                document.querySelectorAll('.invite-button').forEach((element) => {
                    let userId = element.dataset.userId;
                    if (allDestIds.includes(userId)) {
                        element.remove();
                        let invites = JSON.parse(localStorage.getItem('invites')) || {};
                        delete invites[userId];
                        localStorage.setItem('invites', JSON.stringify(invites));
                    }
                });
                closeModal();
            }
        

            function toggleSidebar() {
                
            const sidebar = document.getElementById("sidebar");
            const toggleButton = document.querySelector(".toggle-sidebar-btn");
            const chatContainer = document.getElementById("chatContainer");

            sidebar.classList.toggle("visible");

            if (sidebar.classList.contains("visible")) {

                toggleButton.style.left = "300px";
                chatContainer.classList.add("sidebar-visible");
            } else {

                toggleButton.style.left = "10px";
                chatContainer.classList.remove("sidebar-visible");
                }
            }
        
            async function sendMessage() {
                const fileInput = document.getElementById('fileInput');
                if (fileInput.files.length != 0) {
                    await sendFile();
                }
                
                const targetUserId = currentChatDestUserId;
                const textarea = document.querySelector('.input-textarea');
                const thing = textarea.value;
                addMessageToHistory(`${thing}`, 'client', currentChatNum);

                const message = thing.replace(/ /g, '&nbsp;').replace(/\n/g, '<br>');
                if (!message && !fileInput) {
                    alert("Please enter a message or select a file.");
                    return;
                }
                const encryptedMessage = await encryptMessage(stringToUint8Array(message), sharedSecret[targetUserId]).catch((error) => {
                    console.error("Error encrypting message :", error);
                });
                const signature = signDilithium(new TextEncoder().encode(message));
                socket.emit('send_message', { target_user_id: targetUserId, message: arrayBufferToHex(encryptedMessage), signature: signature});
                
            }
            
            function addMessageToHistory(message, type, chatId, sourceId) {

                if (sourceId == currentChatDestUserId || type == "client") {

                    const messagesContainer = document.querySelector('.messages');
                    const newMessage = document.createElement('div');
                    newMessage.textContent = message.replace(/&nbsp;/g, ' ').replace(/<br>/g, '\n');

                    newMessage.classList.add('message', type);
                    messagesContainer.appendChild(newMessage);
                

                    messagesContainer.scrollTop = messagesContainer.scrollHeight;
                }

                const messageData = {
                    chatid: chatId,
                    message: message,
                    type: type,
                    timestamp: new Date().toISOString(),
                };
            
                saveMessage(messageData).catch((error) => {
                    console.error("Error saving message to database:", error);
                });
                const messageInput = document.querySelector('.input-textarea'); 
                messageInput.value = '';
            }

            function closeChatContainer() {

                const chatContainer = document.getElementById('chatContainer');

                if (chatContainer) {

                    document.body.removeChild(chatContainer);
                    isChatShowed = false;
                }
            }

            function openChatContainer() {

                const chatContainer = document.getElementById('chatContainer');
                const logo = document.getElementById("asciiArt");
                logo.style.display = "none";
                chatContainer.style.display = 'flex';
            }

            function openModal() {
                const modal = document.getElementById("chatModal");
                modal.style.display = "flex";
            }
            
            function closeModal() {
                const modal = document.getElementById("chatModal");
                modal.style.display = "none";

            }

            async function initializeKyber() {
                try {
                    const { Kyber1024 } = await import("https://esm.sh/crystals-kyber-js@1.1.1");
                    window.kyberInstance = new Kyber1024();
                } catch (error) {
                    console.error("Error loading Kyber1024:", error);
                }
            };
            async function submitPassword() {
                if (!pageLoaded) {
                    alert('The page is not fully loaded yet please wait a bit');
                }
                password = document.getElementById('passwordInput').value;
                if (password) {
                    document.getElementById('passwordPopup').style.display = 'none';
                } else {
                    alert('Please enter a password.');
                }
                await decryptSharedSecrets();
                try {
                    await initializeKyber();
                    await setupChatDatabase();
                    displayAllChats();
                    await setupMessageDatabase();
                    await setup();
                    await getOfflineMessages();
                    generateDilithiumKeyPair();
                    setInterval(checkOutdatedMessages, 5000);
                } catch (error) {
                    console.error("Error during initialization:", error);
                }
            }

            function openWebpage() {
                window.open(`/register`, '_blank');
            }
            let socket;
            let sessionUserId = null;
            let currentChatDestUserId;
            let currentChatNum;
            let isChatShowed;
            let chatDb;
            let messageDb;
            let dilithiumPrivateKey;
            let dilithiumPublicKey;
            let areChatsDisplayed = 1;
            let dilithium_keys = {};
            let allDestIds = [];
            let onlineIds = [];
            let PrivateKeyList = {};
            let sharedSecret = {};
            let password;
            let pageLoaded = false;
            window.onload = function() {
                pageLoaded = true;
            }
        