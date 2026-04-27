// Lelins web UI — gestion onglets + appels API streaming (SSE).

document.querySelectorAll('.tab').forEach(btn => {
  btn.addEventListener('click', () => {
    const target = btn.dataset.tab;
    document.querySelectorAll('.tab').forEach(t => t.classList.toggle('active', t === btn));
    document.querySelectorAll('.panel').forEach(p =>
      p.classList.toggle('active', p.dataset.panel === target));
  });
});

// Quand le toggle « Mode rapide » bascule, on adapte la résolution recommandée
// (SDXL Turbo donne ses meilleurs résultats en 512x512). On sauvegarde l'ancienne
// valeur pour la restaurer si on redécoche.
document.querySelectorAll('.fast-toggle input[type="checkbox"]').forEach(cb => {
  cb.addEventListener('change', () => {
    const form = cb.closest('form');
    if (!form) return;
    const w = form.querySelector('input[name="width"]');
    const h = form.querySelector('input[name="height"]');
    if (cb.checked) {
      if (w) { w.dataset.previous = w.value; w.value = 512; }
      if (h) { h.dataset.previous = h.value; h.value = 512; }
    } else {
      if (w && w.dataset.previous) w.value = w.dataset.previous;
      if (h && h.dataset.previous) h.value = h.dataset.previous;
    }
  });
});

function fmtSeconds(s) {
  if (s == null || isNaN(s)) return '';
  if (s < 60) return Math.round(s) + 's';
  const m = Math.floor(s / 60);
  const r = Math.round(s - m * 60);
  return `${m}min ${r.toString().padStart(2, '0')}s`;
}

function renderProgress(target, initialMessage) {
  const el = document.querySelector(`.result[data-result="${target}"]`);
  el.innerHTML = `
    <div class="progress">
      <div class="progress-info">
        <span class="progress-message"><span class="spinner"></span><span class="msg-text">${initialMessage}</span></span>
        <span class="progress-eta"></span>
      </div>
      <div class="progress-bar"><div class="progress-bar-fill" style="width: 0%"></div></div>
      <div class="progress-detail"></div>
    </div>`;
  return {
    msg: el.querySelector('.msg-text'),
    eta: el.querySelector('.progress-eta'),
    fill: el.querySelector('.progress-bar-fill'),
    detail: el.querySelector('.progress-detail'),
    root: el,
  };
}

function showResult(target, data) {
  const el = document.querySelector(`.result[data-result="${target}"]`);
  const url = data.image_url + '?t=' + Date.now();
  el.innerHTML = `
    <img src="${url}" alt="résultat">
    <div class="meta">
      <span>Seed&nbsp;: <strong>${data.seed}</strong></span>
      <span>Durée&nbsp;: <strong>${fmtSeconds(data.elapsed_seconds)}</strong></span>
      ${data.face_lock !== undefined ? `<span>Visage verrouillé&nbsp;: <strong>${data.face_lock ? 'oui' : 'non'}</strong></span>` : ''}
      <a href="${url}" download>Télécharger</a>
    </div>`;
}

function showError(target, message) {
  const el = document.querySelector(`.result[data-result="${target}"]`);
  el.innerHTML = `<div class="status error">Erreur&nbsp;: ${message}</div>`;
}

function handleEvent(target, ui, data) {
  switch (data.type) {
    case 'status':
      ui.msg.textContent = data.message || '';
      ui.detail.textContent = '';
      break;
    case 'ready':
      ui.msg.textContent = data.message || 'Génération en cours…';
      if (data.total) ui.detail.textContent = `0 / ${data.total} pas`;
      break;
    case 'step': {
      const pct = Math.max(0, Math.min(100, data.percent || 0));
      ui.fill.style.width = pct + '%';
      ui.msg.textContent = `Pas ${data.step} / ${data.total}`;
      ui.detail.textContent = `${pct.toFixed(0)}% · écoulé ${fmtSeconds(data.elapsed)}`;
      ui.eta.textContent = data.eta > 0 ? `restant ~${fmtSeconds(data.eta)}` : '';
      break;
    }
    case 'done':
      showResult(target, data);
      break;
    case 'error':
      showError(target, data.message || 'inconnue');
      break;
  }
}

async function streamSubmit(form, url, target, initialMessage) {
  const btn = form.querySelector('button[type="submit"]');
  btn.disabled = true;
  const ui = renderProgress(target, initialMessage);

  try {
    const fd = new FormData(form);
    for (const [k, v] of [...fd.entries()]) {
      if (v === '' || v === null) fd.delete(k);
    }

    const res = await fetch(url, { method: 'POST', body: fd });
    if (!res.ok || !res.body) {
      const detail = await res.text().catch(() => '');
      throw new Error(`HTTP ${res.status} ${detail.slice(0, 200)}`);
    }

    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';

    while (true) {
      const { value, done } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });

      // Découpe par paire de \n\n (séparateur SSE).
      let sep;
      while ((sep = buffer.indexOf('\n\n')) !== -1) {
        const chunk = buffer.slice(0, sep);
        buffer = buffer.slice(sep + 2);

        for (const line of chunk.split('\n')) {
          if (!line.startsWith('data: ')) continue;
          let payload;
          try { payload = JSON.parse(line.slice(6)); }
          catch { continue; }
          handleEvent(target, ui, payload);
          if (payload.type === 'done' || payload.type === 'error') {
            return;
          }
        }
      }
    }
  } catch (e) {
    showError(target, e.message);
  } finally {
    btn.disabled = false;
  }
}

document.getElementById('form-produit').addEventListener('submit', e => {
  e.preventDefault();
  streamSubmit(e.target, '/api/generate-stream', 'produit',
               'Préparation…');
});

document.getElementById('form-portrait').addEventListener('submit', e => {
  e.preventDefault();
  streamSubmit(e.target, '/api/generate-character-stream', 'portrait',
               'Préparation…');
});

document.getElementById('form-tryon').addEventListener('submit', e => {
  e.preventDefault();
  streamSubmit(e.target, '/api/tryon-stream', 'tryon',
               'Préparation…');
});
