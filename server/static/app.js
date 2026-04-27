// Lelins web UI — gestion onglets + appels API.

document.querySelectorAll('.tab').forEach(btn => {
  btn.addEventListener('click', () => {
    const target = btn.dataset.tab;
    document.querySelectorAll('.tab').forEach(t => t.classList.toggle('active', t === btn));
    document.querySelectorAll('.panel').forEach(p =>
      p.classList.toggle('active', p.dataset.panel === target));
  });
});

function showStatus(target, message, isError = false) {
  const el = document.querySelector(`.result[data-result="${target}"]`);
  el.innerHTML = `
    <div class="status${isError ? ' error' : ''}">
      ${isError ? '' : '<div class="spinner"></div>'}
      <span>${message}</span>
    </div>`;
}

function showResult(target, data) {
  const el = document.querySelector(`.result[data-result="${target}"]`);
  const url = data.image_url + '?t=' + Date.now();  // bust le cache
  el.innerHTML = `
    <img src="${url}" alt="résultat">
    <div class="meta">
      <span>Seed&nbsp;: <strong>${data.seed}</strong></span>
      <span>Durée&nbsp;: <strong>${data.elapsed_seconds}s</strong></span>
      ${data.face_lock !== undefined ? `<span>Visage verrouillé&nbsp;: <strong>${data.face_lock ? 'oui' : 'non'}</strong></span>` : ''}
      <a href="${url}" download>Télécharger</a>
    </div>`;
}

async function submitForm(form, url, target, statusMsg) {
  const btn = form.querySelector('button[type="submit"]');
  btn.disabled = true;
  showStatus(target, statusMsg);
  try {
    const fd = new FormData(form);
    // Nettoyer les champs vides — sinon FastAPI les passe en string ""
    for (const [k, v] of [...fd.entries()]) {
      if (v === '' || v === null) fd.delete(k);
    }
    const res = await fetch(url, { method: 'POST', body: fd });
    if (!res.ok) {
      const detail = await res.text();
      throw new Error(`${res.status} — ${detail}`);
    }
    const data = await res.json();
    showResult(target, data);
  } catch (e) {
    showStatus(target, `Erreur : ${e.message}`, true);
  } finally {
    btn.disabled = false;
  }
}

document.getElementById('form-produit').addEventListener('submit', e => {
  e.preventDefault();
  submitForm(e.target, '/api/generate', 'produit',
             'Génération en cours… (premier lancement : 30-60 s pour charger le modèle)');
});

document.getElementById('form-portrait').addEventListener('submit', e => {
  e.preventDefault();
  submitForm(e.target, '/api/generate-character', 'portrait',
             'Portrait en cours… (premier lancement : 30-60 s pour charger le modèle)');
});

document.getElementById('form-tryon').addEventListener('submit', e => {
  e.preventDefault();
  submitForm(e.target, '/api/tryon', 'tryon',
             'Essayage en cours… (premier lancement : 60-120 s pour charger l\'inpainting)');
});
