const chatArea = document.getElementById('chatArea');
const messageInput = document.getElementById('messageInput');
const sendBtn = document.getElementById('sendBtn');
const newSessionBtn = document.getElementById('newSessionBtn');

let isProcessing = false;

messageInput.addEventListener('input', () => {
  messageInput.style.height = 'auto';
  messageInput.style.height = Math.min(messageInput.scrollHeight, 200) + 'px';
});

messageInput.addEventListener('keydown', (e) => {
  if (e.ctrlKey && e.key === 'Enter') {
    e.preventDefault();
    sendMessage();
  }
});

sendBtn.addEventListener('click', sendMessage);

newSessionBtn.addEventListener('click', () => {
  chatArea.innerHTML = '';
  const welcome = document.createElement('div');
  welcome.className = 'welcome';
  welcome.id = 'welcomeScreen';
  welcome.innerHTML = `
    <div class="logo">Hajimi</div>
    <div class="mascot">
      <svg width="64" height="64" viewBox="0 0 64 64" fill="none">
        <rect x="12" y="12" width="40" height="40" rx="4" fill="#d4a574"/>
        <rect x="20" y="22" width="8" height="8" rx="1" fill="#1e1e1e"/>
        <rect x="36" y="22" width="8" height="8" rx="1" fill="#1e1e1e"/>
        <rect x="24" y="38" width="16" height="4" rx="1" fill="#1e1e1e"/>
        <rect x="16" y="44" width="6" height="8" rx="1" fill="#d4a574"/>
        <rect x="42" y="44" width="6" height="8" rx="1" fill="#d4a574"/>
      </svg>
    </div>
    <p class="welcome-text">Local-first AI agent. Ask me to read files, run tests, or edit code.</p>
  `;
  chatArea.appendChild(welcome);
  document.getElementById('sessionTitle').textContent = 'Untitled';
});

async function sendMessage() {
  const text = messageInput.value.trim();
  if (!text || isProcessing) return;

  const welcome = document.getElementById('welcomeScreen');
  if (welcome) welcome.remove();

  addMessage('user', text);
  messageInput.value = '';
  messageInput.style.height = 'auto';
  isProcessing = true;
  sendBtn.disabled = true;

  const thinkingId = addThinking();

  try {
    const response = await callBackend(text);
    removeThinking(thinkingId);
    addMessage('ai', response);
  } catch (err) {
    removeThinking(thinkingId);
    addMessage('ai', 'Error: ' + err.message);
  } finally {
    isProcessing = false;
    sendBtn.disabled = false;
    messageInput.focus();
  }
}

function addMessage(role, text) {
  const div = document.createElement('div');
  div.className = 'message ' + role;
  const avatar = role === 'user' ? 'You' : 'H';
  div.innerHTML = '<div class="message-avatar">' + avatar + '</div><div class="message-body">' + formatText(text) + '</div>';
  chatArea.appendChild(div);
  chatArea.scrollTop = chatArea.scrollHeight;
}

function addThinking() {
  const id = 't' + Date.now();
  const div = document.createElement('div');
  div.className = 'message ai';
  div.id = id;
  div.innerHTML = '<div class="message-avatar">H</div><div class="message-body thinking"><div class="thinking-dot"></div><div class="thinking-dot"></div><div class="thinking-dot"></div></div>';
  chatArea.appendChild(div);
  chatArea.scrollTop = chatArea.scrollHeight;
  return id;
}

function removeThinking(id) {
  const el = document.getElementById(id);
  if (el) el.remove();
}

async function callBackend(text) {
  try {
    const res = await fetch('http://localhost:3000/api/chat', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ message: text })
    });
    if (res.ok) {
      const data = await res.json();
      return data.response || JSON.stringify(data);
    }
  } catch {}

  return "I am Hajimi running in standalone mode. To enable full features, start the API server with:\n\nnode src/foundation/api/server.js";
}

function formatText(text) {
  let html = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
  html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
  html = html.replace(/`(.+?)`/g, '<code>$1</code>');
  html = html.replace(/```([\s\S]*?)```/g, '<pre><code>$1</code></pre>');
  html = html.replace(/\n/g, '<br>');
  return html;
}
