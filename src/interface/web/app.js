import { invoke, Channel } from '@tauri-apps/api/core';

const chatArea = document.getElementById('chatArea');
const messageInput = document.getElementById('messageInput');
const sendBtn = document.getElementById('sendBtn');
const newSessionBtn = document.getElementById('newSessionBtn');
const sessionTitle = document.getElementById('sessionTitle');

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
    <p class="welcome-text">Local-first AI agent. Ask me to read files, run tests, or chat with an LLM.</p>
  `;
  chatArea.appendChild(welcome);
  sessionTitle.textContent = 'Untitled';
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
    const response = await handleCommand(text);
    removeThinking(thinkingId);
    if (response && typeof response === 'string') {
      addMessage('ai', response);
    }
  } catch (err) {
    removeThinking(thinkingId);
    addMessage('ai', '**Error:** ' + (err.message || err));
  } finally {
    isProcessing = false;
    sendBtn.disabled = false;
    messageInput.focus();
  }
}

async function handleCommand(text) {
  const lower = text.toLowerCase();

  // --- Tool-system commands ---
  if (lower === '/tools' || lower === '/list') {
    const tools = await invoke('list_tools');
    return [
      '**Available tools (' + tools.length + '):**',
      '',
      tools.map(t => '- `' + t.name + '` — ' + t.description).join('\n'),
      '',
      'Use `/tool <name> <json-args>` to execute.'
    ].join('\n');
  }

  if (lower.startsWith('/tool ')) {
    const rest = text.slice(6).trim();
    const spaceIdx = rest.indexOf(' ');
    const name = spaceIdx > 0 ? rest.slice(0, spaceIdx) : rest;
    const argsStr = spaceIdx > 0 ? rest.slice(spaceIdx + 1) : '{}';
    let args;
    try {
      args = JSON.parse(argsStr);
    } catch {
      args = { input: argsStr };
    }
    const result = await invoke('execute_tool', { name, args });
    let out = '**Tool:** `' + name + '`\n';
    if (result.stdout) out += '```\n' + result.stdout + '\n```\n';
    if (result.stderr) out += '**stderr:**\n```\n' + result.stderr + '\n```\n';
    out += '**exit:** ' + (result.exit_code ?? 'N/A');
    return out;
  }

  if (lower === '/providers') {
    const providers = await invoke('get_providers');
    return [
      '**LLM Providers:**',
      '',
      providers.map(p => '- `' + p.name + '` — ' + (p.available ? '✅ available' : '❌ not configured') + ' (default: `' + p.defaultModel + '`)').join('\n')
    ].join('\n');
  }

  if (lower.startsWith('/chat ')) {
    const rest = text.slice(6).trim();
    const spaceIdx = rest.indexOf(' ');
    const provider = spaceIdx > 0 ? rest.slice(0, spaceIdx) : 'ollama';
    const prompt = spaceIdx > 0 ? rest.slice(spaceIdx + 1) : rest;
    if (!prompt) return 'Usage: `/chat <provider> <prompt>`';
    await streamChat(provider, prompt);
    return null; // streaming handles its own message rendering
  }

  // --- Legacy file-system commands ---
  if (lower.startsWith('read ') || lower.startsWith('cat ') || lower.startsWith('show ')) {
    const path = text.replace(/^\w+\s+/, '').trim();
    try {
      const content = await invoke('read_file', { path });
      return '**' + path + '**\n```\n' + content + '\n```';
    } catch (e) {
      return 'Cannot read `' + path + '`: ' + e;
    }
  }

  if (lower.startsWith('ls ') || lower.startsWith('dir ') || lower.startsWith('list ')) {
    const path = text.replace(/^\w+\s+/, '').trim() || '.';
    try {
      const entries = await invoke('list_dir', { path });
      return '**' + path + '**\n```\n' + entries.join('\n') + '\n```';
    } catch (e) {
      return 'Cannot list `' + path + '`: ' + e;
    }
  }

  if (lower.startsWith('write ') || lower.startsWith('save ')) {
    const parts = text.replace(/^\w+\s+/, '').split(' ');
    const path = parts[0];
    const content = parts.slice(1).join(' ');
    if (!path || !content) return 'Usage: write &lt;path&gt; &lt;content&gt;';
    try {
      await invoke('write_file', { path, content });
      return 'Saved to `' + path + '`';
    } catch (e) {
      return 'Cannot write `' + path + '`: ' + e;
    }
  }

  if (lower.startsWith('run ') || lower.startsWith('exec ') || lower.startsWith('git ')) {
    const words = text.split(' ');
    const cmd = words[0];
    const args = words.slice(1);
    try {
      const output = await invoke('run_command', { cmd, args });
      return '```\n$ ' + text + '\n' + output + '\n```';
    } catch (e) {
      return '```\n$ ' + text + '\n' + e + '\n```';
    }
  }

  if (lower === 'help' || lower === '?' || lower === '/help') {
    return [
      '**Available commands:**',
      '',
      '**File system:**',
      '- `read &lt;path&gt;` — read file contents',
      '- `write &lt;path&gt; &lt;content&gt;` — write to file',
      '- `ls &lt;path&gt;` — list directory',
      '- `run &lt;command&gt;` — run shell command',
      '',
      '**Tool system:**',
      '- `/tools` — list all 38+ registered tools',
      '- `/tool &lt;name&gt; &lt;json-args&gt;` — execute a tool',
      '',
      '**LLM:**',
      '- `/providers` — show available LLM providers',
      '- `/chat &lt;provider&gt; &lt;prompt&gt;` — stream chat (providers: ollama, anthropic, openai)',
      '',
      'Or type any message to chat with the default LLM (Ollama).'
    ].join('\n');
  }

  // Default: treat as chat with default provider (Ollama)
  await streamChat('ollama', text);
  return null;
}

// ------------------------------------------------------------------
// Streaming chat
// ------------------------------------------------------------------
async function streamChat(provider, prompt) {
  const msgId = 'msg-' + Date.now();
  const div = document.createElement('div');
  div.className = 'message ai';
  div.id = msgId;
  div.innerHTML = '<div class="message-avatar">H</div><div class="message-body"><span class="stream-text"></span><span class="cursor">▋</span></div>';
  chatArea.appendChild(div);
  chatArea.scrollTop = chatArea.scrollHeight;

  const textSpan = div.querySelector('.stream-text');
  const cursor = div.querySelector('.cursor');

  const channel = new Channel();
  channel.onmessage = (ev) => {
    if (ev.error) {
      textSpan.innerHTML += formatText('\n**Error:** ' + ev.error);
      cursor.remove();
      chatArea.scrollTop = chatArea.scrollHeight;
      return;
    }
    if (ev.done) {
      cursor.remove();
      chatArea.scrollTop = chatArea.scrollHeight;
      return;
    }
    textSpan.innerHTML += formatText(ev.chunk);
    chatArea.scrollTop = chatArea.scrollHeight;
  };

  try {
    await invoke('stream_chat', { provider, prompt, onEvent: channel });
  } catch (e) {
    textSpan.innerHTML += formatText('\n**Error:** ' + (e.message || e));
    cursor.remove();
    chatArea.scrollTop = chatArea.scrollHeight;
  }
}

// ------------------------------------------------------------------
// UI helpers
// ------------------------------------------------------------------
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
