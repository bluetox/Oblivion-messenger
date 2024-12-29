import {submitPassword, toggleSidebar, openModal, closeModal, addChat} from '/static/js/utils/uiChatInteract.js';
import {sendMessage} from '/static/js/utils/uiChatInteract.js';
import {removeChat, loadInvites} from '/static/js/utils/utils.js'
import { Kyber1024 } from '/static/js/cdn/kyber.js';

window.socket = null;
window.sessionUserId = null;
window.currentChatDestUserId = null;
window.currentChatNum = null;
window.isChatShowed = false;
window.chatDb = null;
window.messageDb = null;
window.dilithiumPrivateKey = null;
window.dilithiumPublicKey = null;
window.areChatsDisplayed = 1;
window.dilithium_keys = {};
window.kyberInstance = new Kyber1024();
window.kyberPublicKey = null;
window.kyberPrivateKey = null;
window.allDestIds = [];
window.onlineIds = [];
window.PrivateKeyList = {};
window.sharedSecret = {};
window.password = null;
window.pageLoaded = false;

window.onload = function() {
    window.pageLoaded = true;
    const submitButton = document.getElementById('submitButton');
    submitButton.addEventListener('click', submitPassword);

    const sidebarButton = document.getElementById('sidebarButton');
    sidebarButton.addEventListener('click', toggleSidebar);

    const addChatButton = document.getElementById('addChatButton');
    addChatButton.addEventListener('click', openModal);

    const addChatModalCancelButton = document.getElementById('addChatModalCancelButton');
    addChatModalCancelButton.addEventListener('click', closeModal)

    const addChatModalSubmmitButton = document.getElementById('addChatModalSubmmitButton');
    addChatModalSubmmitButton.addEventListener('click', addChat)
    
    const removeChatButton = document.getElementById('removeChatButton');
    removeChatButton.addEventListener('click', removeChat)

    const sendMessageButton = document.getElementById('sendMessageButton');
    sendMessageButton.addEventListener('click', sendMessage);

    const chatInvitesButton = document.getElementById('chatInvitesButton');
    chatInvitesButton.addEventListener('click', loadInvites)
}
