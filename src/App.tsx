import { Routes, Route} from 'react-router-dom';

import './index.css'

import ChatList from './components/ChatList.tsx';
import ChatPage from './components/ChatPage.tsx';

import EnterPassword from './components/Password.tsx'

import SettingsPrivacy from './components/settings/SettingsPrivacy.tsx';
import SettingsMain from './components/settings/SettingsMain.tsx';

import SelectProfile from './components/SelectProfile.tsx';
import AddProfile from './components/AddProfile.tsx';

import AddChat from './components/AddChat.tsx';
import AddPrivateChat from './components/AddPrivateChat.tsx'
import CallPage from './components/CallPage.tsx';
import SettingsGeneral from './components/settings/SettingsGeneral.tsx';

function App() {
  return (
    <div className="flex bg-accent-100 flex-col bg-purple">
      <Routes>
        <Route path="/" element={<SelectProfile />} />
        <Route path="/add" element={<AddProfile />} />
        <Route path="/password" element={<EnterPassword />} />
        
        <Route path="/chat-list" element={<ChatList />} />
        <Route path="/chat-page" element={<ChatPage />} />

        <Route path="/settings-main" element={<SettingsMain />} />
        <Route path="/settings-privacy" element={<SettingsPrivacy />} />
        <Route path="/settings-general" element={<SettingsGeneral />} />

        <Route path="/add-chat" element={<AddChat />} />
        <Route path="/add-private-chat" element={<AddPrivateChat />} />
        
        <Route path="/call-page" element={<CallPage />} />
      </Routes>
    </div>
  );
}


export default App;
