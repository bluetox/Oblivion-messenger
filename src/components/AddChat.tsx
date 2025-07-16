import { useNavigate } from 'react-router-dom';

export default function AddChat() {
    const navigate = useNavigate();
    return (
        <div className="min-h-screen w-full flex flex-col justify-center bg-neutral-950 text-neutral-100 opacity-0 animate-fade-in">
            <div className="flex items-center font bg-neutral-900 border-neutral-800 border-b">
                <svg className="m-4 size-8" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" onClick={() => {navigate('/chat-list')}}>
                  <path fillRule="evenodd" d="M7.28 7.72a.75.75 0 0 1 0 1.06l-2.47 2.47H21a.75.75 0 0 1 0 1.5H4.81l2.47 2.47a.75.75 0 1 1-1.06 1.06l-3.75-3.75a.75.75 0 0 1 0-1.06l3.75-3.75a.75.75 0 0 1 1.06 0Z" clipRule="evenodd" />
                </svg>
                <div className="m-4 font-bold">Add Chat</div>
            </div>
            <div className="flex-1">
                <div className=" bg-neutral-800 px-10 py-6 border-b border-neutral-700" onClick={() => navigate('/add-private-chat')}>
                    <div className="">
                        Add private chat
                    </div>
                    <div className="text-xs mt-1 text-neutral-400">
                        Chat securely
                    </div>
                </div>
                <div className="p-4 bg-neutral-800 px-10 py-8 border-b border-neutral-700" onClick={() => navigate('/add-private-chat')}>
                    <div className="">
                        Add groupe chat
                    </div>
                    <div className="text-xs mt-1 text-neutral-400">
                        Chat with friends
                    </div>
                </div>
            </div>
        </div>
    )
}