import { useNavigate, useLocation } from 'react-router-dom';
import { useEffect, useRef, useState } from "react";
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
  isPermissionGranted,
  sendNotification,
} from '@tauri-apps/plugin-notification';


interface Message {
  chat_id: string;
  content: string;
  message_id: string;
  message_type: string;
  sender_id: string;
}

interface MessageNotif {
  source: string;
  message: string;
  chatId: string;
}


function ChatPage() {
  
  const navigate = useNavigate();
  const location = useLocation();
  const {chatId, chatName} = location.state || {};
  const [inputText, setInputText] = useState("");

  const [messages, setMessages] = useState<Message[]>([]);

  const messagesEndRef = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    async function fetchMessages() {
      const messages: Message[] = await invoke("get_messages", { chatId: chatId });
      setMessages(messages);
    }

    fetchMessages();
  }, [chatId]);

  useEffect(() => {
    const unlistenPromise = listen<MessageNotif>('received-message', async (event) => {
      if (typeof event.payload === 'string') {
        const payload: MessageNotif = JSON.parse(event.payload);

        if (chatId === payload.chatId) {
            setMessages(prev => [
              ...prev,
              {
                chat_id: payload.chatId,
                content: payload.message,
                message_id: Date.now().toString(), // or generate UUID
                message_type: "received",
                sender_id: payload.source,
              },
            ]);
        } else {
          let permissionGranted = await isPermissionGranted();
          if (permissionGranted) {
            if (permissionGranted) {
              sendNotification({ title: 'New Message', body: payload.message });
            }
          }
        }

        
      } else {
        console.warn('Unexpected payload type:', event.payload);
      }
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);


  const sendMessage = async () => {
    const text = inputText.trim();

    if (!text) return;
    await invoke("send_message", {
        chatId: chatId,
        messageString: text
      });
      
    setMessages((prev) => [
      ...prev,
      {
        chat_id: chatId,
        content: text,
        message_id: Date.now().toString(), // or a UUID
        message_type: "sent",
        sender_id: "me",
      },
    ]);

    setInputText("");
  };

useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  return (
    <div
      className="flex flex-col min-h-screen max-h-screen w-screen bg-neutral-950 text-neutral-100 opacity-0 animate-fade-in" 
    >
      {/* Header */}
      <div className="flex items-center bg-neutral-900 px-4 py-4">
        <svg
          className="size-8 mr-4 cursor-pointer text-neutral-500"
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="currentColor"
          onClick={() => navigate('/chat-list')}
        >
          <path
            fillRule="evenodd"
            d="M7.28 7.72a.75.75 0 0 1 0 1.06l-2.47 2.47H21a.75.75 0 0 1 0 1.5H4.81l2.47 2.47a.75.75 0 1 1-1.06 1.06l-3.75-3.75a.75.75 0 0 1 0-1.06l3.75-3.75a.75.75 0 0 1 1.06 0Z"
            clipRule="evenodd"
          />
        </svg>

        <div className="w-10 h-10 rounded-full overflow-hidden">
          <img
            src="https://t4.ftcdn.net/jpg/04/31/64/75/240_F_431647519_usrbQ8Z983hTYe8zgA7t1XVc5fEtqcpa.jpg"
            alt="circle"
            className="w-full h-full object-cover"
          />
        </div>

        <div className="mx-4 text-lg font-semibold">{chatName}</div>

        <div className="ml-auto flex flex-row items-center text-neutral-500">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="currentColor"
            className="size-8 mx-2 cursor-pointer"
          >
            <path d="M4.5 4.5a3 3 0 0 0-3 3v9a3 3 0 0 0 3 3h8.25a3 3 0 0 0 3-3v-9a3 3 0 0 0-3-3H4.5ZM19.94 18.75l-2.69-2.69V7.94l2.69-2.69c.944-.945 2.56-.276 2.56 1.06v11.38c0 1.336-1.616 2.005-2.56 1.06Z" />
          </svg>
          <svg
            onClick={() => navigate('/call-page',{ state: { chatId: chatId }})}
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="currentColor"
            className="size-7 mx-2 cursor-pointer"
          >
            <path
              fillRule="evenodd"
              d="M1.5 4.5a3 3 0 0 1 3-3h1.372c.86 0 1.61.586 1.819 1.42l1.105 4.423a1.875 1.875 0 0 1-.694 1.955l-1.293.97c-.135.101-.164.249-.126.352a11.285 11.285 0 0 0 6.697 6.697c.103.038.25.009.352-.126l.97-1.293a1.875 1.875 0 0 1 1.955-.694l4.423 1.105c.834.209 1.42.959 1.42 1.82V19.5a3 3 0 0 1-3 3h-2.25C8.552 22.5 1.5 15.448 1.5 6.75V4.5Z"
              clipRule="evenodd"
            />
          </svg>
        </div>
      </div>

      {/* Chat messages area */}
      <div className="flex-1 overflow-y-auto p-4 space-y-2 scroll-smooth scrollbar-thin scrollbar-thumb-neutral-700 scrollbar-track-transparent">
        {/* Messages will go here */}
        {messages.map((msg) => (
          <div
            key={msg.message_id} // ✅ Fix here too
            className={`flex ${msg.message_type === "sent" ? "justify-end" : "justify-start"}`}
          >
            <div
              className={`max-w-[70%] px-4 py-2 rounded-2xl text-sm shadow-sm break-words whitespace-normal ${
                msg.message_type === "sent"
                  ? "bg-gradient-to-br from-primary-700 to-primary-500 text-white"
                  : "bg-gradient-to-br from-neutral-800 to-neutral-600 text-neutral-200"
              }`}
            >
              {msg.content}
            </div>
          </div>
        ))}
        <div ref={messagesEndRef} />
      </div>

      {/* Footer input bar */}
      <div className="sticky bottom-0 flex px-4 items-center flex-row py-4 bg-zinc-900">
  <div className="w-full bg-zinc-800 rounded-full flex items-center px-4">
    <textarea
      value={inputText}
      onChange={(e) => setInputText(e.target.value)}
      placeholder="Type a message..."
      rows={1}
      className="bg-zinc-800 text-white rounded-md px-4 py-2 flex-1 outline-none resize-none"
    />
  </div>

        <div className="rounded-full bg-primary-500 ml-4" onClick={() => {sendMessage()}}>
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="size-10 p-2">
            <path d="M3.478 2.404a.75.75 0 0 0-.926.941l2.432 7.905H13.5a.75.75 0 0 1 0 1.5H4.984l-2.432 7.905a.75.75 0 0 0 .926.94 60.519 60.519 0 0 0 18.445-8.986.75.75 0 0 0 0-1.218A60.517 60.517 0 0 0 3.478 2.404Z" />
          </svg>
            
        </div>
      </div>
    </div>
  );
}

export default ChatPage;
