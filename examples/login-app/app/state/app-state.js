import {
  createManagedUser,
  createTask,
  deleteManagedUser,
  deleteTask,
  getTask,
  listTasks,
  listUsers,
  loginUser,
  registerUser,
  updateManagedUser,
  updateTask
} from '../lib/api.js';
import { taskStatusValue } from '../lib/options.js';

function blankTaskComposer() {
  return {
    title: '',
    description: ''
  };
}

function blankTaskDetailForm() {
  return {
    title: '',
    description: '',
    completed: 'pending'
  };
}

function blankUserForm() {
  return {
    id: '',
    name: '',
    email: '',
    password: ''
  };
}

function rerender() {
  App.requestRender();
}

function updateValue(target, key, value) {
  if (target[key] === value) {
    return false;
  }

  target[key] = value;
  return true;
}

function sortTasks(items) {
  return items
    .slice()
    .sort((left, right) => String(right.updatedAt || '').localeCompare(String(left.updatedAt || '')));
}

function sortUsers(items) {
  return items
    .slice()
    .sort((left, right) => String(left.name || '').localeCompare(String(right.name || '')));
}

function resetProtectedState() {
  state.tasks = [];
  state.taskDetail = null;
  state.users = [];
  state.forms.createTask = blankTaskComposer();
  state.forms.taskDetail = blankTaskDetailForm();
  state.forms.user = blankUserForm();
  state.filters.tasks = 'all';
  state.filters.users = 'all';
  state.loaded.tasksToken = '';
  state.loaded.usersToken = '';
  state.loaded.taskId = '';
  state.pendingTaskId = '';
  state.userModal.visible = false;
  state.userModal.mode = 'create';
  state.userModal.targetId = '';
}

function applyNotice(tone, text) {
  state.notice = {
    tone,
    text
  };
}

function upsertTask(task) {
  const withoutCurrent = state.tasks.filter((item) => item.id !== task.id);
  state.tasks = sortTasks([...withoutCurrent, task]);
}

function upsertUser(user) {
  const withoutCurrent = state.users.filter((item) => item.id !== user.id);
  state.users = sortUsers([...withoutCurrent, user]);
}

function seedTaskDetail(task) {
  state.taskDetail = task;
  state.forms.taskDetail = {
    title: task.title || '',
    description: task.description || '',
    completed: taskStatusValue(task.completed)
  };
  state.loaded.taskId = task.id;
}

function currentToken() {
  return state.session.token;
}

function currentUserId() {
  return state.session.user?.id || '';
}

function showConfirmationAlert({
  title,
  description,
  confirmText = 'Confirm',
  onConfirm
}) {
  Alert({
    title,
    description,
    primaryButtonText: confirmText,
    primaryButtonOnClick: () => {
      if (typeof onConfirm === 'function') {
        onConfirm();
      }
    },
    secondaryButtonText: 'Cancel'
  });
}

export const state = {
  session: {
    token: '',
    user: null
  },
  notice: null,
  authModal: null,
  forms: {
    login: {
      email: '',
      password: ''
    },
    register: {
      name: '',
      email: '',
      password: ''
    },
    createTask: blankTaskComposer(),
    taskDetail: blankTaskDetailForm(),
    user: blankUserForm()
  },
  filters: {
    tasks: 'all',
    users: 'all'
  },
  tasks: [],
  taskDetail: null,
  users: [],
  loading: {
    auth: false,
    tasks: false,
    task: false,
    users: false,
    taskCreate: false,
    taskSave: false,
    userSave: false,
    deletingTaskId: '',
    deletingUserId: ''
  },
  loaded: {
    tasksToken: '',
    usersToken: '',
    taskId: ''
  },
  pendingTaskId: '',
  userModal: {
    visible: false,
    mode: 'create',
    targetId: ''
  }
};

export function isAuthenticated() {
  return Boolean(state.session.token && state.session.user);
}

export function clearNotice() {
  if (!state.notice) {
    return;
  }

  state.notice = null;
  rerender();
}

export function openAuthModal(mode) {
  state.authModal = mode;
  state.notice = null;
  rerender();
}

export function closeAuthModal() {
  if (!state.authModal) {
    return;
  }

  state.authModal = null;
  rerender();
}

export function setLoginField(field, value) {
  if (updateValue(state.forms.login, field, value)) {
    rerender();
  }
}

export function setRegisterField(field, value) {
  if (updateValue(state.forms.register, field, value)) {
    rerender();
  }
}

export function setTaskComposerField(field, value) {
  if (updateValue(state.forms.createTask, field, value)) {
    rerender();
  }
}

export function setTaskDetailField(field, value) {
  if (updateValue(state.forms.taskDetail, field, value)) {
    rerender();
  }
}

export function setTaskFilter(value) {
  if (updateValue(state.filters, 'tasks', value)) {
    rerender();
  }
}

export function setUserFilter(value) {
  if (updateValue(state.filters, 'users', value)) {
    rerender();
  }
}

export function openUserModal(mode, user = null) {
  state.userModal.visible = true;
  state.userModal.mode = mode;
  state.userModal.targetId = user?.id || '';
  state.forms.user = {
    id: user?.id || '',
    name: user?.name || '',
    email: user?.email || '',
    password: ''
  };
  state.notice = null;
  rerender();
}

export function closeUserModal() {
  state.userModal.visible = false;
  state.userModal.mode = 'create';
  state.userModal.targetId = '';
  state.forms.user = blankUserForm();
  rerender();
}

export function setUserFormField(field, value) {
  if (updateValue(state.forms.user, field, value)) {
    rerender();
  }
}

export async function submitLogin(route) {
  if (state.loading.auth) {
    return;
  }

  const email = state.forms.login.email.trim().toLowerCase();
  const password = state.forms.login.password;

  if (!email || !password) {
    applyNotice('error', 'Enter an email and password to continue.');
    rerender();
    return;
  }

  state.loading.auth = true;
  state.notice = null;
  rerender();

  try {
    const payload = await loginUser({ email, password });
    state.session.token = payload.token;
    state.session.user = payload.user;
    resetProtectedState();
    state.authModal = null;
    state.forms.login.password = '';
    applyNotice('success', `Logged in as ${payload.user.name}.`);
    route.navigate('/tasks');
  } catch (error) {
    applyNotice('error', error.message || 'Could not log in.');
  } finally {
    state.loading.auth = false;
    rerender();
  }
}

export async function submitRegister(route) {
  if (state.loading.auth) {
    return;
  }

  const name = state.forms.register.name.trim();
  const email = state.forms.register.email.trim().toLowerCase();
  const password = state.forms.register.password;

  if (!name || !email || password.length < 6) {
    applyNotice('error', 'Name, email, and a 6-character password are required.');
    rerender();
    return;
  }

  state.loading.auth = true;
  state.notice = null;
  rerender();

  try {
    await registerUser({ name, email, password });
    const payload = await loginUser({ email, password });
    state.session.token = payload.token;
    state.session.user = payload.user;
    resetProtectedState();
    state.authModal = null;
    state.forms.register = {
      name: '',
      email,
      password: ''
    };
    applyNotice('success', `Account created for ${payload.user.name}.`);
    route.navigate('/tasks');
  } catch (error) {
    applyNotice('error', error.message || 'Could not create the account.');
  } finally {
    state.loading.auth = false;
    rerender();
  }
}

export function logout(route) {
  state.session.token = '';
  state.session.user = null;
  state.authModal = null;
  resetProtectedState();
  applyNotice('info', 'You have been logged out.');
  route.navigate('/');
  rerender();
}

export function requestLogout(route) {
  if (!isAuthenticated()) {
    return;
  }

  showConfirmationAlert({
    title: 'Sign out?',
    description: 'You will return to the landing page and protected data will reset.',
    confirmText: 'Sign out',
    onConfirm: () => logout(route)
  });
}

export function ensureTasksLoaded() {
  if (!currentToken() || state.loading.tasks || state.loaded.tasksToken === currentToken()) {
    return;
  }

  void refreshTasks();
}

export async function refreshTasks() {
  const token = currentToken();

  if (!token || state.loading.tasks) {
    return;
  }

  state.loading.tasks = true;
  rerender();

  try {
    const payload = await listTasks(token);

    if (currentToken() !== token) {
      return;
    }

    state.tasks = sortTasks(payload.tasks || []);
    state.loaded.tasksToken = token;

    if (state.taskDetail?.id) {
      const nextTask = state.tasks.find((task) => task.id === state.taskDetail.id);

      if (nextTask) {
        seedTaskDetail(nextTask);
      }
    }
  } catch (error) {
    applyNotice('error', error.message || 'Could not load tasks.');
  } finally {
    state.loading.tasks = false;
    rerender();
  }
}

export async function createTaskFromComposer() {
  const token = currentToken();
  const title = state.forms.createTask.title.trim();
  const description = state.forms.createTask.description.trim();

  if (!token || state.loading.taskCreate) {
    return;
  }

  if (!title) {
    applyNotice('error', 'Enter a task title before saving.');
    rerender();
    return;
  }

  state.loading.taskCreate = true;
  state.notice = null;
  rerender();

  try {
    const payload = await createTask(token, {
      title,
      ...(description ? { description } : {})
    });

    if (currentToken() !== token) {
      return;
    }

    upsertTask(payload.task);
    state.loaded.tasksToken = token;
    state.forms.createTask = blankTaskComposer();
    applyNotice('success', 'Task created.');
  } catch (error) {
    applyNotice('error', error.message || 'Could not create the task.');
  } finally {
    state.loading.taskCreate = false;
    rerender();
  }
}

export function ensureTaskDetail(taskId) {
  if (!currentToken() || !taskId) {
    return;
  }

  const cachedTask = state.tasks.find((task) => task.id === taskId);

  if (cachedTask && state.taskDetail?.id !== taskId) {
    seedTaskDetail(cachedTask);
  }

  if (
    state.loaded.taskId === taskId ||
    (state.loading.task && state.pendingTaskId === taskId)
  ) {
    return;
  }

  void loadTaskDetail(taskId);
}

export async function loadTaskDetail(taskId) {
  const token = currentToken();

  if (!token || !taskId) {
    return;
  }

  state.loading.task = true;
  state.pendingTaskId = taskId;
  rerender();

  try {
    const payload = await getTask(token, taskId);

    if (currentToken() !== token || state.pendingTaskId !== taskId) {
      return;
    }

    upsertTask(payload.task);
    seedTaskDetail(payload.task);
  } catch (error) {
    applyNotice('error', error.message || 'Could not load the task.');
  } finally {
    if (state.pendingTaskId === taskId) {
      state.pendingTaskId = '';
    }

    state.loading.task = false;
    rerender();
  }
}

export async function saveTaskDetail(route) {
  const token = currentToken();
  const taskId = state.taskDetail?.id;
  const title = state.forms.taskDetail.title.trim();
  const description = state.forms.taskDetail.description.trim();

  if (!token || !taskId || state.loading.taskSave) {
    return;
  }

  if (!title) {
    applyNotice('error', 'Task title cannot be empty.');
    rerender();
    return;
  }

  state.loading.taskSave = true;
  state.notice = null;
  rerender();

  try {
    const payload = await updateTask(token, taskId, {
      title,
      description: description ? description : null,
      completed: state.forms.taskDetail.completed === 'completed'
    });

    if (currentToken() !== token) {
      return;
    }

    upsertTask(payload.task);
    seedTaskDetail(payload.task);
    applyNotice('success', 'Task updated.');

    if (route?.query?.mode !== 'view') {
      route.replace(`/tasks/${taskId}?mode=view`);
    }
  } catch (error) {
    applyNotice('error', error.message || 'Could not update the task.');
  } finally {
    state.loading.taskSave = false;
    rerender();
  }
}

export async function removeTask(taskId, route, nextPath = '') {
  const token = currentToken();

  if (!token || !taskId || state.loading.deletingTaskId === taskId) {
    return;
  }

  state.loading.deletingTaskId = taskId;
  state.notice = null;
  rerender();

  try {
    await deleteTask(token, taskId);

    if (currentToken() !== token) {
      return;
    }

    state.tasks = state.tasks.filter((task) => task.id !== taskId);

    if (state.taskDetail?.id === taskId) {
      state.taskDetail = null;
      state.forms.taskDetail = blankTaskDetailForm();
      state.loaded.taskId = '';
    }

    applyNotice('success', 'Task deleted.');

    if (route && nextPath) {
      route.navigate(nextPath);
    }
  } catch (error) {
    applyNotice('error', error.message || 'Could not delete the task.');
  } finally {
    state.loading.deletingTaskId = '';
    rerender();
  }
}

export function requestTaskRemoval(taskId, route, nextPath = '') {
  const task = state.tasks.find((item) => item.id === taskId) || state.taskDetail;
  const taskName = task?.title || 'this task';

  showConfirmationAlert({
    title: 'Delete task?',
    description: `This permanently deletes "${taskName}". This action cannot be undone.`,
    confirmText: 'Delete',
    onConfirm: () => {
      void removeTask(taskId, route, nextPath);
    }
  });
}

export function ensureUsersLoaded() {
  if (!currentToken() || state.loading.users || state.loaded.usersToken === currentToken()) {
    return;
  }

  void refreshUsers();
}

export async function refreshUsers() {
  const token = currentToken();

  if (!token || state.loading.users) {
    return;
  }

  state.loading.users = true;
  rerender();

  try {
    const payload = await listUsers(token);

    if (currentToken() !== token) {
      return;
    }

    state.users = sortUsers(payload.users || []);
    state.loaded.usersToken = token;
  } catch (error) {
    applyNotice('error', error.message || 'Could not load users.');
  } finally {
    state.loading.users = false;
    rerender();
  }
}

export async function saveUserModal() {
  const token = currentToken();
  const name = state.forms.user.name.trim();
  const email = state.forms.user.email.trim().toLowerCase();
  const password = state.forms.user.password;
  const isEditing = state.userModal.mode === 'edit';

  if (!token || state.loading.userSave) {
    return;
  }

  if (!name || !email) {
    applyNotice('error', 'Name and email are required.');
    rerender();
    return;
  }

  if (state.userModal.mode === 'create' && password.length < 6) {
    applyNotice('error', 'A new user needs a password with at least 6 characters.');
    rerender();
    return;
  }

  state.loading.userSave = true;
  state.notice = null;
  rerender();

  try {
    const payload =
      isEditing
        ? await updateManagedUser(token, state.userModal.targetId, {
            name,
            email,
            ...(password.trim() ? { password } : {})
          })
        : await createManagedUser(token, {
            name,
            email,
            password
          });

    if (currentToken() !== token) {
      return;
    }

    upsertUser(payload.user);
    state.loaded.usersToken = token;

    if (payload.user.id === currentUserId()) {
      state.session.user = payload.user;
    }

    state.userModal.visible = false;
    state.userModal.mode = 'create';
    state.userModal.targetId = '';
    state.forms.user = blankUserForm();
    applyNotice('success', isEditing ? 'User updated.' : 'User added.');
  } catch (error) {
    applyNotice('error', error.message || 'Could not save the user.');
  } finally {
    state.loading.userSave = false;
    rerender();
  }
}

export async function removeUser(userId) {
  const token = currentToken();

  if (!token || !userId || state.loading.deletingUserId === userId) {
    return;
  }

  if (userId === currentUserId()) {
    applyNotice('error', 'A user cannot delete itself.');
    rerender();
    return;
  }

  state.loading.deletingUserId = userId;
  state.notice = null;
  rerender();

  try {
    await deleteManagedUser(token, userId);

    if (currentToken() !== token) {
      return;
    }

    state.users = state.users.filter((user) => user.id !== userId);
    applyNotice('success', 'User deleted.');
  } catch (error) {
    applyNotice('error', error.message || 'Could not delete the user.');
  } finally {
    state.loading.deletingUserId = '';
    rerender();
  }
}

export function requestUserRemoval(userId) {
  if (userId === currentUserId()) {
    applyNotice('error', 'A user cannot delete itself.');
    rerender();
    return;
  }

  const target = state.users.find((user) => user.id === userId);
  const label = target ? `${target.name} (${target.email})` : 'this user';

  showConfirmationAlert({
    title: 'Delete user?',
    description: `This permanently deletes ${label}. This action cannot be undone.`,
    confirmText: 'Delete',
    onConfirm: () => {
      void removeUser(userId);
    }
  });
}
