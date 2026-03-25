function pad(value) {
  return String(value).padStart(2, '0');
}

export function formatDateTime(value) {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return 'Unknown time';
  }

  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export function summarizeText(value, maxLength = 96) {
  const text = String(value || '').trim();

  if (!text) {
    return 'No description yet.';
  }

  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, maxLength - 3)}...`;
}

export function taskStatusLabel(completed) {
  return completed ? 'Completed' : 'Open';
}

export function countLabel(count, singular, plural = `${singular}s`) {
  return `${count} ${count === 1 ? singular : plural}`;
}
