import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';

export default function AddPrivateChat() {
  const navigate = useNavigate();
  const [chatName, setChatName] = useState('');
  const [dstId, setDstId] = useState('');
  const [error, setError] = useState('');

  const isValidHash = (id: string) => /^[a-fA-F0-9]{64}$/.test(id);
  const isFormValid = chatName.trim() !== '' && isValidHash(dstId.trim());

  const handleSubmit = async () => {
    if (!isFormValid) return;

    try {
      await invoke("create_private_chat", {name: chatName, dstUserId: dstId.trim()});
      navigate('/chat-list');
    } catch (err) {
      setError('Failed to add chat. Please try again.');
      console.error(err);
    }
  };

  return (
    <div className="min-h-screen w-full flex flex-col bg-neutral-950 text-neutral-100 animate-fade-in">
      <div className="flex items-center bg-neutral-900 border-b border-neutral-800">
        <svg
          className="m-4 size-8 cursor-pointer"
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="currentColor"
          onClick={() => navigate('/add-chat')}
        >
          <path
            fillRule="evenodd"
            d="M7.28 7.72a.75.75 0 0 1 0 1.06l-2.47 2.47H21a.75.75 0 0 1 0 1.5H4.81l2.47 2.47a.75.75 0 1 1-1.06 1.06l-3.75-3.75a.75.75 0 0 1 0-1.06l3.75-3.75a.75.75 0 0 1 1.06 0Z"
            clipRule="evenodd"
          />
        </svg>
        <div className="m-4 font-bold">Add Private Chat</div>
      </div>

      <div className="flex flex-col gap-4 p-6">
        <label className="flex flex-col">
          <span className="text-sm text-neutral-300 mb-1">Chat Name</span>
          <input
            className="bg-neutral-900 border border-neutral-700 rounded px-3 py-2 text-sm focus:outline-none focus:border-neutral-500"
            value={chatName}
            onChange={(e) => setChatName(e.target.value)}
            placeholder="e.g. Alice"
          />
        </label>

        <label className="flex flex-col">
          <span className="text-sm text-neutral-300 mb-1">Destination ID (256-bit hash)</span>
          <input
            className="bg-neutral-900 border border-neutral-700 rounded px-3 py-2 text-sm font-mono focus:outline-none focus:border-neutral-500"
            value={dstId}
            onChange={(e) => setDstId(e.target.value)}
            placeholder="64-character hex string"
          />
        </label>

        {error && <div className="text-xs text-red-400 mt-2">{error}</div>}

        <button
          onClick={handleSubmit}
          disabled={!isFormValid}
          className={`mt-4 rounded px-4 py-2 text-sm font-semibold transition ${
            isFormValid
              ? 'bg-primary-700 hover:bg-primary-600'
              : 'bg-neutral-800 text-neutral-500 cursor-not-allowed'
          }`}
        >
          Add Chat
        </button>
      </div>
    </div>
  );
}
