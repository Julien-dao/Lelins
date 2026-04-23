export function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  // Delay to ensure the browser started the download before revoking.
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function downloadDataUrl(dataUrl: string, filename: string) {
  const a = document.createElement('a');
  a.href = dataUrl;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
}

export function canShareFiles(files: File[]): boolean {
  const nav = navigator as Navigator & { canShare?: (data: ShareData) => boolean };
  if (!nav.canShare || typeof navigator.share !== 'function') return false;
  try {
    return nav.canShare({ files });
  } catch {
    return false;
  }
}

export async function shareFiles(files: File[], text: string, title = 'Lelins Studio') {
  if (!navigator.share) throw new Error('Web Share API non disponible sur cet appareil.');
  await navigator.share({ files, text, title });
}
