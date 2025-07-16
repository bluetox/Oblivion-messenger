import { useNavigate } from 'react-router-dom';
import { useState, useEffect } from "react";
import { invoke } from '@tauri-apps/api/core';

type privacy = {
    encryption: string,
    signature: string,
    key_exchange: string,
    chat_deletion_timer: number
}

type settings = {
    name: string,
    user_id: string,
    privacy_settings: privacy,
}

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text)
    .then(() => {
      console.log("Copied to clipboard!");
    })
    .catch(err => {
      console.error("Failed to copy: ", err);
    });
}

export default function SettingsMain() {
    const navigate = useNavigate();
    const [settings, setSettings] = useState<settings>();

    useEffect(() => {
      (async () => {
        let settings = await invoke<settings>("get_profile_settings");
        console.log(settings);
        setSettings(settings);
      })();
    }, []);
    
  return (
    <div className="flex flex-col w-screen h-screen  bg-neutral-950 text-neutral-100 opacity-0 animate-fade-in">
        <div className="flex items-center font bg-neutral-900 border-b border-neutral-800">
            <svg className="m-4 size-8" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" onClick={() => {navigate('/chat-list')}}>
              <path fillRule="evenodd" d="M7.28 7.72a.75.75 0 0 1 0 1.06l-2.47 2.47H21a.75.75 0 0 1 0 1.5H4.81l2.47 2.47a.75.75 0 1 1-1.06 1.06l-3.75-3.75a.75.75 0 0 1 0-1.06l3.75-3.75a.75.75 0 0 1 1.06 0Z" clipRule="evenodd" />
            </svg>
            <div className="m-4 font-bold">Parameters</div>
        </div>
        <div className="flex flex-row">
            <div className="flex m-4">
                <div className="w-20 h-20 rounded-full bg-primary-600 border-2 border-primary-400 flex items-center justify-center text-xl text-white font-bold mb-2">
                  {settings?.name[0]?.toUpperCase() || '?'}
                </div>
                <div className="flex m-4 ml-8 flex-col">
                    <div className='text-2xl'>
                        {settings?.name ?? "Loading..."}
                    </div>
                    <div className="text-xs text-neutral-400 flex flex-row" onClick={() => copyToClipboard(settings?.user_id ?? "")}>
                        {settings?.user_id.slice(0,32) ?? "Loading..."}
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="ml-2 size-4">
                          <path d="M7.5 3.375c0-1.036.84-1.875 1.875-1.875h.375a3.75 3.75 0 0 1 3.75 3.75v1.875C13.5 8.161 14.34 9 15.375 9h1.875A3.75 3.75 0 0 1 21 12.75v3.375C21 17.16 20.16 18 19.125 18h-9.75A1.875 1.875 0 0 1 7.5 16.125V3.375Z" />
                          <path d="M15 5.25a5.23 5.23 0 0 0-1.279-3.434 9.768 9.768 0 0 1 6.963 6.963A5.23 5.23 0 0 0 17.25 7.5h-1.875A.375.375 0 0 1 15 7.125V5.25ZM4.875 6H6v10.125A3.375 3.375 0 0 0 9.375 19.5H16.5v1.125c0 1.035-.84 1.875-1.875 1.875h-9.75A1.875 1.875 0 0 1 3 20.625V7.875C3 6.839 3.84 6 4.875 6Z" />
                        </svg>

                    </div>
                </div>
            </div>
        </div>
        <div className="flex-1">
            <div className="flex flex-row h-16 items-center">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="size-8 mx-6 text-neutral-500">
                    <path fillRule="evenodd" d="M18.685 19.097A9.723 9.723 0 0 0 21.75 12c0-5.385-4.365-9.75-9.75-9.75S2.25 6.615 2.25 12a9.723 9.723 0 0 0 3.065 7.097A9.716 9.716 0 0 0 12 21.75a9.716 9.716 0 0 0 6.685-2.653Zm-12.54-1.285A7.486 7.486 0 0 1 12 15a7.486 7.486 0 0 1 5.855 2.812A8.224 8.224 0 0 1 12 20.25a8.224 8.224 0 0 1-5.855-2.438ZM15.75 9a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0Z" clipRule="evenodd" />
                </svg>
                <div className=''> Account</div>
            </div>
            <div className="flex flex-row h-16 items-center">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="size-8 mx-6 text-neutral-500">
                  <path d="M21.731 2.269a2.625 2.625 0 0 0-3.712 0l-1.157 1.157 3.712 3.712 1.157-1.157a2.625 2.625 0 0 0 0-3.712ZM19.513 8.199l-3.712-3.712-12.15 12.15a5.25 5.25 0 0 0-1.32 2.214l-.8 2.685a.75.75 0 0 0 .933.933l2.685-.8a5.25 5.25 0 0 0 2.214-1.32L19.513 8.2Z" />
                </svg>
                <div className=''>Appearance</div>
            </div>
            <div className="flex flex-row h-16 items-center" onClick={() => {navigate('/settings-privacy')} }>
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="size-8 mx-6 text-neutral-500">
                  <path fillRule="evenodd" d="M12 1.5a5.25 5.25 0 0 0-5.25 5.25v3a3 3 0 0 0-3 3v6.75a3 3 0 0 0 3 3h10.5a3 3 0 0 0 3-3v-6.75a3 3 0 0 0-3-3v-3c0-2.9-2.35-5.25-5.25-5.25Zm3.75 8.25v-3a3.75 3.75 0 1 0-7.5 0v3h7.5Z" clipRule="evenodd" />
                </svg>

                <div className=''>Privacy</div>
            </div>
            <div className="flex flex-row h-16 items-center" onClick={() => {navigate('/')} }> 
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="size-8 mx-6 text-neutral-500">
                  <path fillRule="evenodd" d="M16.5 3.75a1.5 1.5 0 0 1 1.5 1.5v13.5a1.5 1.5 0 0 1-1.5 1.5h-6a1.5 1.5 0 0 1-1.5-1.5V15a.75.75 0 0 0-1.5 0v3.75a3 3 0 0 0 3 3h6a3 3 0 0 0 3-3V5.25a3 3 0 0 0-3-3h-6a3 3 0 0 0-3 3V9A.75.75 0 1 0 9 9V5.25a1.5 1.5 0 0 1 1.5-1.5h6ZM5.78 8.47a.75.75 0 0 0-1.06 0l-3 3a.75.75 0 0 0 0 1.06l3 3a.75.75 0 0 0 1.06-1.06l-1.72-1.72H15a.75.75 0 0 0 0-1.5H4.06l1.72-1.72a.75.75 0 0 0 0-1.06Z" clipRule="evenodd" />
                </svg>


                <div className=''>Log Out</div>
            </div>
        </div>
    </div>
  );
}
