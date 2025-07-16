import { useNavigate } from 'react-router-dom';
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from "@tauri-apps/api/event";

type Chat = {
  chat_id: string;
  chat_name: string;
  chat_type: string;
};


function ChatList() {
  const navigate = useNavigate();
  const [chats, setChats] = useState<Chat[]>([]);
  const [disconnected, setDisconnected] = useState(false); // <-- new

  useEffect(() => {
    invoke<Chat[]>("get_chats").then((c) => {
      setChats(c);
    });
  }, []);

  useEffect(() => {
    const unlistenPromise = listen("core-error", (event: any) => {
      if (event.payload === "DISCONNECTED") {
        console.log("Disconnected from core");
        setDisconnected(true); // <-- trigger overlay
      }
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);



  return (
    <div className="flex flex-col h-[100dvh] w-full bg-neutral-950 text-neutral-100 opacity-0 animate-fade-in">
      {/* Disconnection overlay*/}
      {disconnected && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-neutral-950 text-white text-center px-4">
          <div className="bg-neutral-900 border border-red-500 text-red-500 rounded-xl p-6 shadow-lg animate-fade-in">
            <p className="text-lg font-semibold">Disconnected from the core service</p>
            <p className="text-sm mt-2 text-neutral-400">Please restart the app or check your connection.</p>
          </div>
        </div>
      )}

      {/* Top bar */}
      <div className="flex items-center justify-between h-16 px-4 border-b border-neutral-800 bg-neutral-900">
        <div className="flex items-center space-x-3">
          <svg
            onClick={() => navigate('/settings-main')}
            className="h-7 w-7 text-neutral-500 cursor-pointer"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="currentColor"
          >
            <path
              fillRule="evenodd"
              d="M11.828 2.25c-.916 0-1.699.663-1.85 1.567l-.091.549a.798.798 0 0 1-.517.608 7.45 7.45 0 0 0-.478.198.798.798 0 0 1-.796-.064l-.453-.324a1.875 1.875 0 0 0-2.416.2l-.243.243a1.875 1.875 0 0 0-.2 2.416l.324.453a.798.798 0 0 1 .064.796 7.448 7.448 0 0 0-.198.478.798.798 0 0 1-.608.517l-.55.092a1.875 1.875 0 0 0-1.566 1.849v.344c0 .916.663 1.699 1.567 1.85l.549.091c.281.047.508.25.608.517.06.162.127.321.198.478a.798.798 0 0 1-.064.796l-.324.453a1.875 1.875 0 0 0 .2 2.416l.243.243c.648.648 1.67.733 2.416.2l.453-.324a.798.798 0 0 1 .796-.064c.157.071.316.137.478.198.267.1.47.327.517.608l.092.55c.15.903.932 1.566 1.849 1.566h.344c.916 0 1.699-.663 1.85-1.567l.091-.549a.798.798 0 0 1 .517-.608 7.52 7.52 0 0 0 .478-.198.798.798 0 0 1 .796.064l.453.324a1.875 1.875 0 0 0 2.416-.2l.243-.243c.648-.648.733-1.67.2-2.416l-.324-.453a.798.798 0 0 1-.064-.796c.071-.157.137-.316.198-.478.1-.267.327-.47.608-.517l.55-.091a1.875 1.875 0 0 0 1.566-1.85v-.344c0-.916-.663-1.699-1.567-1.85l-.549-.091a.798.798 0 0 1-.608-.517 7.507 7.507 0 0 0-.198-.478.798.798 0 0 1 .064-.796l.324-.453a1.875 1.875 0 0 0-.2-2.416l-.243-.243a1.875 1.875 0 0 0-2.416-.2l-.453.324a.798.798 0 0 1-.796.064 7.462 7.462 0 0 0-.478-.198.798.798 0 0 1-.517-.608l-.091-.55a1.875 1.875 0 0 0-1.85-1.566h-.344ZM12 15.75a3.75 3.75 0 1 0 0-7.5 3.75 3.75 0 0 0 0 7.5Z"
              clipRule="evenodd"
            />
          </svg>
          <div className="font-semibold text-lg">Oblivion</div>
        </div>

        <div className="flex items-center space-x-3">
          <svg
            className="h-7 w-7 cursor-pointer text-neutral-500"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="currentColor"
          >
            <path
              fillRule="evenodd"
              d="M10.5 3.75a6.75 6.75 0 1 0 0 13.5 6.75 6.75 0 0 0 0-13.5ZM2.25 10.5a8.25 8.25 0 1 1 14.59 5.28l4.69 4.69a.75.75 0 1 1-1.06 1.06l-4.69-4.69A8.25 8.25 0 0 1 2.25 10.5Z"
              clipRule="evenodd"
            />
          </svg>

          <svg
            className="h-7 w-7 cursor-pointer text-neutral-500"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="currentColor"
          >
            <path
              fillRule="evenodd"
              d="M10.5 6a1.5 1.5 0 1 1 3 0 1.5 1.5 0 0 1-3 0Zm0 6a1.5 1.5 0 1 1 3 0 1.5 1.5 0 0 1-3 0Zm0 6a1.5 1.5 0 1 1 3 0 1.5 1.5 0 0 1-3 0Z"
              clipRule="evenodd"
            />
          </svg>
        </div>
      </div>

      {/* Chat List */}
      <div className="flex flex-col flex-1 overflow-y-auto py-2">
        {chats.map((chat) => (
          <div
            key={chat.chat_id}
            onClick={async () => {
              if (await invoke("has_shared_secret", { chatId: chat.chat_id }) == true) {
                console.log("going");
                navigate('/chat-page',{ state: { chatId: chat.chat_id, chatName: chat.chat_name} });
              } else {
                invoke("establish_ss", { chatId: chat.chat_id  }).then(()=> console.log("done"));
              }
              
            }}
            className="mx-4 my-2 px-4 py-3 rounded-xl bg-neutral-900 border border-neutral-800 hover:bg-neutral-800 transition-colors cursor-pointer"
          >
            <div className="flex items-center">
              <div className="w-12 h-12 rounded-full bg-gradient-to-br from-neutral-700 to-neutral-800" />
              <div className="flex flex-col ml-4">
                <div className="mb-1 font-medium">{chat.chat_name}</div>
                <div className="text-sm text-neutral-400 truncate">
                  {chat.chat_id.slice(0, 25) + " . . ."}
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Floating Action Button */}
      <div
        onClick={() => navigate('/add-chat')}
        className="fixed right-6 bottom-8 size-14 rounded-full bg-primary-500 shadow-lg hover:bg-primary-600 transition-all duration-150 flex justify-center items-center cursor-pointer"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          strokeWidth={1.5}
          stroke="currentColor"
          className="size-7 text-white"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L6.832 19.82a4.5 4.5 0 0 1-1.897 1.13l-2.685.8.8-2.685a4.5 4.5 0 0 1 1.13-1.897L16.863 4.487Zm0 0L19.5 7.125"
          />
        </svg>
      </div>
    </div>
  );
}

export default ChatList;
