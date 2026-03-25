const DEFAULT_API_BASE_URL = 'http://127.0.0.1:3000';

function trimTrailingSlash(value) {
  return String(value || '').replace(/\/+$/, '');
}

export function getApiBaseUrl() {
  return trimTrailingSlash(globalThis.__LOGIN_APP_API_BASE_URL__ || DEFAULT_API_BASE_URL);
}

function buildHeaders(token, includesBody) {
  const headers = {};

  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }

  if (includesBody) {
    headers['Content-Type'] = 'application/json';
  }

  return headers;
}

function parseBody(text) {
  if (!text) {
    return {};
  }

  try {
    return JSON.parse(text);
  } catch (_error) {
    return { message: text };
  }
}

function normalizeErrorMessage(error) {
  const message = String(error?.message || error || 'Request failed.').trim();

  if (message.includes('401')) {
    return 'Your session is missing or expired. Login again.';
  }

  if (message.includes('403')) {
    return 'This action is not allowed for the current user.';
  }

  if (message.includes('404')) {
    return 'The requested record could not be found.';
  }

  if (message.includes('409')) {
    return 'A record with these details already exists.';
  }

  if (message.includes('500')) {
    return 'The API failed while handling the request.';
  }

  return message || 'Request failed.';
}

async function request(path, options = {}) {
  const method = options.method || 'GET';
  const hasBody = options.body !== undefined;
  const requestOptions = {
    method,
    headers: buildHeaders(options.token, hasBody)
  };

  if (hasBody) {
    requestOptions.body = options.body;
  }

  try {
    const responseText = await fetch(`${getApiBaseUrl()}${path}`, requestOptions);
    return parseBody(responseText);
  } catch (error) {
    throw new Error(normalizeErrorMessage(error));
  }
}

export function registerUser(input) {
  return request('/users', {
    method: 'POST',
    body: input
  });
}

export function loginUser(input) {
  return request('/login', {
    method: 'POST',
    body: input
  });
}

export function listTasks(token) {
  return request('/tasks', { token });
}

export function createTask(token, input) {
  return request('/tasks', {
    method: 'POST',
    token,
    body: input
  });
}

export function getTask(token, taskId) {
  return request(`/tasks/${encodeURIComponent(taskId)}`, { token });
}

export function updateTask(token, taskId, input) {
  return request(`/tasks/${encodeURIComponent(taskId)}`, {
    method: 'PUT',
    token,
    body: input
  });
}

export function deleteTask(token, taskId) {
  return request(`/tasks/${encodeURIComponent(taskId)}`, {
    method: 'DELETE',
    token
  });
}

export function listUsers(token) {
  return request('/admin/users', { token });
}

export function createManagedUser(token, input) {
  return request('/admin/users', {
    method: 'POST',
    token,
    body: input
  });
}

export function updateManagedUser(token, userId, input) {
  return request(`/admin/users/${encodeURIComponent(userId)}`, {
    method: 'PUT',
    token,
    body: input
  });
}

export function deleteManagedUser(token, userId) {
  return request(`/admin/users/${encodeURIComponent(userId)}`, {
    method: 'DELETE',
    token
  });
}
