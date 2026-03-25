export const TASK_FILTER_OPTIONS = [
  { label: 'All tasks', value: 'all' },
  { label: 'Open only', value: 'open' },
  { label: 'Completed', value: 'completed' }
];

export const TASK_STATUS_OPTIONS = [
  { label: 'Pending', value: 'pending' },
  { label: 'Completed', value: 'completed' }
];

export const USER_SCOPE_OPTIONS = [
  { label: 'Everyone', value: 'all' },
  { label: 'My account', value: 'self' },
  { label: 'Others', value: 'others' }
];

export function matchesTaskFilter(filter, task) {
  if (filter === 'open') {
    return !task.completed;
  }

  if (filter === 'completed') {
    return task.completed;
  }

  return true;
}

export function matchesUserScope(scope, user, currentUserId) {
  if (scope === 'self') {
    return user.id === currentUserId;
  }

  if (scope === 'others') {
    return user.id !== currentUserId;
  }

  return true;
}

export function taskStatusValue(completed) {
  return completed ? 'completed' : 'pending';
}
