import  {clearChat, base64ToBuffer} from '/static/js/utils/utils.js'
import {openChatContainer} from '/static/js/utils/uiChatInteract.js'
import { stringToUint8Array } from '/static/js/utils/convertions.js';
import {encryptData, decryptMessage, getDecryptionKey, encryptMessage} from '/static/js/utils/encryption.js'

export async function setupChatDatabase() {

    chatDb = await openDatabase("chatIndex", 1, (db) => {
        if (!db.objectStoreNames.contains("users")) {

            const objectStore = db.createObjectStore("users", { keyPath: "id", autoIncrement: true });

            objectStore.createIndex("name", "name", { unique: false });
            objectStore.createIndex("user_id", "user_id", { unique: true });
            objectStore.createIndex("timestamp", "timestamp");
        }
    });
}
export async function setupMessageDatabase() {

    messageDb = await openDatabase("Messages", 1, (db) => {
        if (!db.objectStoreNames.contains("messages")) {

            const objectStore = db.createObjectStore("messages", { keyPath: "id", autoIncrement: true });

            objectStore.createIndex("chatid", "chatid", { unique: false });
            objectStore.createIndex("message", "message", { unique: false });
            objectStore.createIndex("type", "type");
            objectStore.createIndex("timestamp", "timestamp");
        }
    });
}
export function openDatabase(dbName, version, setupCallback) {
    return new Promise((resolve, reject) => {

        const request = indexedDB.open(dbName, version);
        
        request.onupgradeneeded = (event) => {

            const db = event.target.result;
            setupCallback(db);
        };
        
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject("Failed to open database");
    });
}
export async function saveChat(user) {
    if (!chatDb) {

        console.error("chatDb is not initialized");
        return
    }

    const transaction = chatDb.transaction("users", "readwrite");
    const objectStore = transaction.objectStore("users");
    const request = objectStore.add(user);

    return new Promise((resolve, reject) => {
        request.onsuccess = () => {

            const chatId = request.result;
            
            resolve(chatId);
        };

        request.onerror = (event) => {

            console.error("Error adding chat:", event.target.error);
            reject(event.target.error);
        };
    });
}
export async function saveMessage(data) {
    return new Promise(async (resolve, reject) => {
        if (!messageDb) {
            console.error("messageDb is not initialized");
            reject("Database not initialized");
            return;
        }

        try {
            const messageArray = stringToUint8Array(data.message);
            const destUserId = await findUserIdById(data.chatid);
            const externalKey = encryptionExternalKey[destUserId];
            const bufferKey = base64ToBuffer(externalKey);
            const firstEncryption = await encryptMessage(messageArray, bufferKey);
            const encryptedData = await encryptData(firstEncryption, password);
            data.message = encryptedData;
            const transaction = messageDb.transaction("messages", "readwrite");
            const objectStore = transaction.objectStore("messages");

            const request = objectStore.add(data);

            request.onsuccess = () => {
                resolve();
            };

            request.onerror = (event) => {
                console.error("Error adding message:", event.target.error);
                reject(event.target.error);
            };
        } catch (error) {
            console.error("Error encrypting data:", error);
            reject(error);
        }
    });
}
export async function displayAllChats() {
    if (!chatDb) {
        console.error("chatDb is not initialized");
        return;
    }

    const transaction = chatDb.transaction("users", "readonly");
    const objectStore = transaction.objectStore("users");
    const cursorRequest = objectStore.openCursor();

    cursorRequest.onsuccess = (event) => {
        const cursor = event.target.result;

        if (cursor) {
            const { id, user_id, name, timestamp } = cursor.value;
            const newButton = document.createElement('button');

            allDestIds.push(user_id);
            newButton.textContent = name;
            newButton.classList.add('sidebar-button');
            newButton.dataset.chatid = id;
            newButton.dataset.destId = user_id;
            newButton.dataset.timestamp = timestamp;
            newButton.addEventListener('click', async () => {
                    currentChatDestUserId = user_id;
                    currentChatNum = id;
                    if (!sharedSecret[user_id]) {
                        let publicKey, privateKey;
                        [publicKey, privateKey] = await kyberInstance.generateKeyPair();
                        PrivateKeyList[user_id] = privateKey;
                        socket.emit('append_KyberKey', { public_key: Array.from(publicKey), target_user_id: currentChatDestUserId });
                        
                    }
                    socket.emit('dilithium_key', {key: dilithiumPublicKey.toHex(), 'dest_id' : currentChatDestUserId});
                    clearChat();
                    openChatContainer();
                    encryptionExternalKey[user_id] = await getDecryptionKey(user_id);
                    loadMessages(id);
                }); 

            const sidebar = document.querySelector('.sidebar');
            sidebar.appendChild(newButton);
            cursor.continue();
        }
    };

    cursorRequest.onerror = (event) => {
        console.error("Error retrieving chats:", event.target.error);
    };
}
export async function loadMessages(chatId) {
    return new Promise((resolve, reject) => {
        if (!chatId) {
            console.error("Invalid chatId:", chatId);
            return reject("Invalid chatId");
        }
        if (!messageDb) {
            console.error("messageDb is not initialized");
            return reject("Database not initialized");
        }

        const transaction = messageDb.transaction("messages", "readonly");
        const objectStore = transaction.objectStore("messages");
        const chatIndex = objectStore.index("chatid");
        const keyRange = IDBKeyRange.only(chatId);
        const cursorRequest = chatIndex.openCursor(keyRange, "next");

        const messagesContainer = document.querySelector('.messages');
        if (!messagesContainer) {
            console.error("Messages container not found");
            return reject("Messages container not found");
        }

        const messagePromises = [];

        cursorRequest.onsuccess = async (event) => {
            const cursor = event.target.result;
            if (!cursor) {
                try {
                    await Promise.all(messagePromises);
                    resolve();
                } catch (error) {
                    reject(error);
                }
                return;
            }

            const { message, type } = cursor.value;

            const processMessage = (async () => {
                try {
                    const destUserId = await findUserIdById(chatId);
                    if (!encryptionExternalKey[destUserId]) {
                        let response = await getDecryptionKey(destUserId);
                        if (response == null) {
                            console.log("Message decryption has been disabled !");
                            return;
                        }
                        encryptionExternalKey[destUserId] = response;
                    }
                    const key = encryptionExternalKey[destUserId]
                    const bufferKey = base64ToBuffer(key);
                    const firstDecrypt = await decryptMessage(message, password, "file");
                    const finalMessageBuffer = await decryptMessage(firstDecrypt, bufferKey, "message");

                    const decryptedMessage = new TextDecoder().decode(finalMessageBuffer);

                    const newMessage = document.createElement('div');
                    newMessage.textContent = decryptedMessage.replace(/&nbsp;/g, ' ').replace(/<br>/g, '\n');
                    newMessage.classList.add('message', type);

                    return newMessage;
                } catch (error) {
                    console.error("Error processing message:", error);
                    throw error;
                }
            })();

            messagePromises.push(
                processMessage.then((newMessage) => {
                    if (newMessage) {
                        messagesContainer.appendChild(newMessage);
                        messagesContainer.scrollTop = messagesContainer.scrollHeight;
                    }
                })
            );

            cursor.continue();
        };

        cursorRequest.onerror = (event) => {
            console.error("Error loading messages:", event.target.error);
            reject("Error loading messages for chatId: " + chatId);
        };
    });
}

export async function removeChatFromDb() {
    if (!chatDb) {
        console.error("chatDb is not initialized");
        return;
    }
    const transaction = chatDb.transaction("users", "readwrite");
    const objectStore = transaction.objectStore("users");

    const request = objectStore.delete(currentChatNum);

    return new Promise((resolve, reject) => {
        request.onsuccess = () => {
            resolve(true);
        };

        request.onerror = (event) => {
            console.error("Error removing chat:", event.target.error);
            reject(event.target.error);
        };
    });
}
export function findIdByUserId(targetUserId) {
    return new Promise((resolve, reject) => {
        const transaction = chatDb.transaction("users", 'readonly');
        const store = transaction.objectStore('users');

        const index = store.index('user_id');
        const getRequest = index.get(targetUserId);

        getRequest.onsuccess = () => {
            if (getRequest.result) {
                resolve(getRequest.result.id);
            } else {
                reject('User not found.');
            }
        };

        getRequest.onerror = () => {
            reject('Error fetching data.');
        };
    });
}
export function findUserIdById(targetId) {
    return new Promise((resolve, reject) => {
        if (!chatDb) {
            reject('Database is not initialized.');
            return;
        }
        const transaction = chatDb.transaction('users', 'readonly');
        const store = transaction.objectStore('users');

        const getRequest = store.get(targetId);

        getRequest.onsuccess = () => {
            const result = getRequest.result;
            if (result && result.user_id) {
                resolve(result.user_id);
            } else {
                console.warn(`No record found for id: ${targetId}`);
                reject(`No user found for id: ${targetId}`);
            }
        };

        getRequest.onerror = (event) => {
            console.error('Request error:', event.target.error);
            reject('Error fetching data from IndexedDB.');
        };

        transaction.onerror = (event) => {
            console.error('Transaction error:', event.target.error);
            reject('Transaction error while fetching data.');
        };
    });
}
export function checkOutdatedMessages() {
    if (!messageDb) {
        console.error("chatDb is not initialized");
        return;
    }

    const transaction = messageDb.transaction("messages", "readwrite");
    const objectStore = transaction.objectStore("messages");

    const currentTime = new Date().getTime();
    const outdatedThreshold = localStorage.getItem('deletionTimer') * 60000;
    if (!outdatedThreshold) {
        return;
    }

    const cursorRequest = objectStore.openCursor();

    cursorRequest.onsuccess = function(event) {
        const cursor = event.target.result;

        if (cursor) {
            const message = cursor.value;
            const messageTimestamp = message.timestamp;
            const date = new Date(messageTimestamp);
            if (currentTime - date > outdatedThreshold) {
                console.log(`Message ID ${cursor.key} is outdated.`);
                objectStore.delete(cursor.key);
            }

            cursor.continue();
        } else {
            console.log("No more messages to check.");
        }
    };

    cursorRequest.onerror = function(event) {
        console.error("Error iterating through users/messages:", event.target.error);
    };
}
