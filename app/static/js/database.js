async function setupChatDatabase() {

    chatDb = await openDatabase("chatIndex", 1, (db) => {
        if (!db.objectStoreNames.contains("users")) {

            const objectStore = db.createObjectStore("users", { keyPath: "id", autoIncrement: true });

            objectStore.createIndex("name", "name", { unique: false });
            objectStore.createIndex("user_id", "user_id", { unique: true });
            objectStore.createIndex("timestamp", "timestamp");
        }
    });
}

async function setupMessageDatabase() {

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

function openDatabase(dbName, version, setupCallback) {
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

async function saveChat(user) {
    if (!chatDb) {

        console.error("chatDb is not initialized");
        reject("Database not initialized");
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

function saveMessage(data) {
    return new Promise((resolve, reject) => {
        if (!messageDb) {

            console.error("messageDb is not initialized");
            reject("Database not initialized");
            return;
        }

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
    });
}

async function displayAllChats() {
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

            allChatIds.push(user_id);
            newButton.textContent = name;
            newButton.classList.add('sidebar-button');
            newButton.dataset.chatid = id;
            newButton.dataset.timestamp = timestamp;
            newButton.onclick = () => {

               
                CurrentChatId = user_id;
                CurrentChatIndex = id;
                socket.emit('append_KyberKey', { public_key: Array.from(kyberPublicKey), target_user_id: CurrentChatId });

                clearChat();
                openChatContainer();
                loadMessages(id);
            };

            const sidebar = document.querySelector('.sidebar');
            sidebar.appendChild(newButton);
            cursor.continue();
        }
    };

    cursorRequest.onerror = (event) => {
        console.error("Error retrieving chats:", event.target.error);
    };
}

function loadMessages(chatId) {
    return new Promise((resolve, reject) => {
        if (!chatId) {

            console.error("Invalid chatId:", chatId);
            reject("Invalid chatId");
            return;
        }
        if (!messageDb) {

            console.error("messageDb is not initialized");
            reject("Database not initialized");
            return;
        }

        const transaction = messageDb.transaction("messages", "readonly");
        const objectStore = transaction.objectStore("messages");
        const chatIndex = objectStore.index("chatid");

        const keyRange = IDBKeyRange.only(chatId);
        const cursorRequest = chatIndex.openCursor(keyRange, "next");

        cursorRequest.onsuccess = (event) => {
            const cursor = event.target.result;
            const messagesContainer = document.querySelector('.messages');

            if (!cursor) {
                return;
            }

            do {
                const { chatid, timestamp, message, type } = cursor.value;
                const newMessage = document.createElement('div');
                newMessage.textContent = message;
                newMessage.classList.add('message', type);
                

                messagesContainer.appendChild(newMessage);
                messagesContainer.scrollTop = messagesContainer.scrollHeight;

                cursor.continue();
            } while (cursor && cursor.result);
        };

        cursorRequest.onerror = () => {
            reject("Error loading messages for: ",chatId);
        };
    });
}


setupChatDatabase().then(displayAllChats);
setupMessageDatabase();
