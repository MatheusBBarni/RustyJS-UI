import { Elysia, t } from 'elysia';

type UserRecord = {
  id: string;
  name: string;
  email: string;
  password: string;
  createdAt: string;
};

type PublicUser = Omit<UserRecord, 'password'>;

type TaskRecord = {
  id: string;
  ownerId: string;
  title: string;
  description: string | null;
  completed: boolean;
  createdAt: string;
  updatedAt: string;
};

const users = new Map<string, UserRecord>();
const usersByEmail = new Map<string, string>();
const sessions = new Map<string, string>();
const tasks = new Map<string, TaskRecord>();

const createUserBody = t.Object({
  name: t.String({ minLength: 1 }),
  email: t.String({ format: 'email' }),
  password: t.String({ minLength: 6 })
});

const loginBody = t.Object({
  email: t.String({ format: 'email' }),
  password: t.String({ minLength: 1 })
});

const createTaskBody = t.Object({
  title: t.String({ minLength: 1 }),
  description: t.Optional(t.String())
});

const updateTaskBody = t.Object({
  title: t.Optional(t.String({ minLength: 1 })),
  description: t.Optional(t.Union([t.String(), t.Null()])),
  completed: t.Optional(t.Boolean())
});

const updateUserBody = t.Object({
  name: t.Optional(t.String({ minLength: 1 })),
  email: t.Optional(t.String({ format: 'email' })),
  password: t.Optional(t.String({ minLength: 6 }))
});

const taskParams = t.Object({
  id: t.String()
});

const userParams = t.Object({
  id: t.String()
});

const normalizeEmail = (email: string) => email.trim().toLowerCase();

const toPublicUser = ({ password: _password, ...user }: UserRecord): PublicUser => user;

const getBearerToken = (authorization?: string | null) => {
  if (!authorization) {
    return null;
  }

  const [scheme, token] = authorization.split(' ');

  if (scheme?.toLowerCase() !== 'bearer' || !token) {
    return null;
  }

  return token.trim();
};

const getCurrentUser = (authorization?: string | null) => {
  const token = getBearerToken(authorization);

  if (!token) {
    return null;
  }

  const userId = sessions.get(token);

  if (!userId) {
    return null;
  }

  return users.get(userId) ?? null;
};

const getTaskForUser = (taskId: string, userId: string) => {
  const task = tasks.get(taskId);

  if (!task || task.ownerId !== userId) {
    return null;
  }

  return task;
};

const getUserForMutation = (userId: string) => users.get(userId) ?? null;

const removeUserSessions = (userId: string) => {
  for (const [token, sessionUserId] of sessions.entries()) {
    if (sessionUserId === userId) {
      sessions.delete(token);
    }
  }
};

const removeUserTasks = (userId: string) => {
  for (const [taskId, task] of tasks.entries()) {
    if (task.ownerId === userId) {
      tasks.delete(taskId);
    }
  }
};

export const app = new Elysia()
  .get('/', () => ({
    name: 'login-app/rest-api',
    status: 'ok',
    routes: [
      'POST /users',
      'POST /login',
      'GET /admin/users',
      'POST /admin/users',
      'PUT /admin/users/:id',
      'PATCH /admin/users/:id',
      'DELETE /admin/users/:id',
      'GET /tasks',
      'POST /tasks',
      'GET /tasks/:id',
      'PUT /tasks/:id',
      'PATCH /tasks/:id',
      'DELETE /tasks/:id'
    ]
  }))
  .post(
    '/users',
    ({ body, set, status }) => {
      const email = normalizeEmail(body.email);

      if (usersByEmail.has(email)) {
        return status(409, {
          message: 'A user with this email already exists.'
        });
      }

      const now = new Date().toISOString();
      const user: UserRecord = {
        id: crypto.randomUUID(),
        name: body.name.trim(),
        email,
        password: body.password,
        createdAt: now
      };

      users.set(user.id, user);
      usersByEmail.set(email, user.id);
      set.status = 201;

      return {
        message: 'User created.',
        user: toPublicUser(user)
      };
    },
    { body: createUserBody }
  )
  .get('/admin/users', ({ headers, status }) => {
    const currentUser = getCurrentUser(headers.authorization);

    if (!currentUser) {
      return status(401, {
        message: 'Missing or invalid bearer token.'
      });
    }

    return {
      users: Array.from(users.values()).map(toPublicUser)
    };
  })
  .post(
    '/admin/users',
    ({ body, headers, set, status }) => {
      const currentUser = getCurrentUser(headers.authorization);

      if (!currentUser) {
        return status(401, {
          message: 'Missing or invalid bearer token.'
        });
      }

      const email = normalizeEmail(body.email);

      if (usersByEmail.has(email)) {
        return status(409, {
          message: 'A user with this email already exists.'
        });
      }

      const now = new Date().toISOString();
      const user: UserRecord = {
        id: crypto.randomUUID(),
        name: body.name.trim(),
        email,
        password: body.password,
        createdAt: now
      };

      users.set(user.id, user);
      usersByEmail.set(email, user.id);
      set.status = 201;

      return {
        message: 'User created.',
        user: toPublicUser(user)
      };
    },
    { body: createUserBody }
  )
  .patch(
    '/admin/users/:id',
    ({ body, params, headers, status }) => {
      const currentUser = getCurrentUser(headers.authorization);

      if (!currentUser) {
        return status(401, {
          message: 'Missing or invalid bearer token.'
        });
      }

      const user = getUserForMutation(params.id);

      if (!user) {
        return status(404, {
          message: 'User not found.'
        });
      }

      if (body.email !== undefined) {
        const normalizedEmail = normalizeEmail(body.email);
        const existingUserId = usersByEmail.get(normalizedEmail);

        if (existingUserId && existingUserId !== user.id) {
          return status(409, {
            message: 'A user with this email already exists.'
          });
        }

        usersByEmail.delete(user.email);
        user.email = normalizedEmail;
        usersByEmail.set(normalizedEmail, user.id);
      }

      if (body.name !== undefined) {
        user.name = body.name.trim();
      }

      if (body.password !== undefined) {
        user.password = body.password;
      }

      return {
        message: 'User updated.',
        user: toPublicUser(user)
      };
    },
    { body: updateUserBody, params: userParams }
  )
  .put(
    '/admin/users/:id',
    ({ body, params, headers, status }) => {
      const currentUser = getCurrentUser(headers.authorization);

      if (!currentUser) {
        return status(401, {
          message: 'Missing or invalid bearer token.'
        });
      }

      const user = getUserForMutation(params.id);

      if (!user) {
        return status(404, {
          message: 'User not found.'
        });
      }

      if (body.email !== undefined) {
        const normalizedEmail = normalizeEmail(body.email);
        const existingUserId = usersByEmail.get(normalizedEmail);

        if (existingUserId && existingUserId !== user.id) {
          return status(409, {
            message: 'A user with this email already exists.'
          });
        }

        usersByEmail.delete(user.email);
        user.email = normalizedEmail;
        usersByEmail.set(normalizedEmail, user.id);
      }

      if (body.name !== undefined) {
        user.name = body.name.trim();
      }

      if (body.password !== undefined) {
        user.password = body.password;
      }

      return {
        message: 'User updated.',
        user: toPublicUser(user)
      };
    },
    { body: updateUserBody, params: userParams }
  )
  .delete(
    '/admin/users/:id',
    ({ params, headers, status }) => {
      const currentUser = getCurrentUser(headers.authorization);

      if (!currentUser) {
        return status(401, {
          message: 'Missing or invalid bearer token.'
        });
      }

      if (currentUser.id === params.id) {
        return status(403, {
          message: 'You cannot delete yourself.'
        });
      }

      const user = getUserForMutation(params.id);

      if (!user) {
        return status(404, {
          message: 'User not found.'
        });
      }

      users.delete(user.id);
      usersByEmail.delete(user.email);
      removeUserSessions(user.id);
      removeUserTasks(user.id);

      return {
        message: 'User deleted.'
      };
    },
    { params: userParams }
  )
  .post(
    '/login',
    ({ body, status }) => {
      const email = normalizeEmail(body.email);
      const userId = usersByEmail.get(email);
      const user = userId ? users.get(userId) : null;

      if (!user || user.password !== body.password) {
        return status(401, {
          message: 'Invalid email or password.'
        });
      }

      const token = crypto.randomUUID();
      sessions.set(token, user.id);

      return {
        message: 'Login successful.',
        token,
        user: toPublicUser(user)
      };
    },
    { body: loginBody }
  )
  .get('/tasks', ({ headers, status }) => {
    const currentUser = getCurrentUser(headers.authorization);

    if (!currentUser) {
      return status(401, {
        message: 'Missing or invalid bearer token.'
      });
    }

    return {
      tasks: Array.from(tasks.values()).filter((task) => task.ownerId === currentUser.id)
    };
  })
  .post(
    '/tasks',
    ({ body, headers, set, status }) => {
      const currentUser = getCurrentUser(headers.authorization);

      if (!currentUser) {
        return status(401, {
          message: 'Missing or invalid bearer token.'
        });
      }

      const now = new Date().toISOString();
      const task: TaskRecord = {
        id: crypto.randomUUID(),
        ownerId: currentUser.id,
        title: body.title.trim(),
        description: body.description?.trim() || null,
        completed: false,
        createdAt: now,
        updatedAt: now
      };

      tasks.set(task.id, task);
      set.status = 201;

      return {
        message: 'Task created.',
        task
      };
    },
    { body: createTaskBody }
  )
  .get(
    '/tasks/:id',
    ({ params, headers, status }) => {
      const currentUser = getCurrentUser(headers.authorization);

      if (!currentUser) {
        return status(401, {
          message: 'Missing or invalid bearer token.'
        });
      }

      const task = getTaskForUser(params.id, currentUser.id);

      if (!task) {
        return status(404, {
          message: 'Task not found.'
        });
      }

      return { task };
    },
    { params: taskParams }
  )
  .patch(
    '/tasks/:id',
    ({ body, params, headers, status }) => {
      const currentUser = getCurrentUser(headers.authorization);

      if (!currentUser) {
        return status(401, {
          message: 'Missing or invalid bearer token.'
        });
      }

      const task = getTaskForUser(params.id, currentUser.id);

      if (!task) {
        return status(404, {
          message: 'Task not found.'
        });
      }

      const updatedTask: TaskRecord = {
        ...task,
        title: body.title?.trim() ?? task.title,
        description:
          body.description === undefined
            ? task.description
            : body.description === null
              ? null
              : body.description.trim(),
        completed: body.completed ?? task.completed,
        updatedAt: new Date().toISOString()
      };

      tasks.set(task.id, updatedTask);

      return {
        message: 'Task updated.',
        task: updatedTask
      };
    },
    { body: updateTaskBody, params: taskParams }
  )
  .put(
    '/tasks/:id',
    ({ body, params, headers, status }) => {
      const currentUser = getCurrentUser(headers.authorization);

      if (!currentUser) {
        return status(401, {
          message: 'Missing or invalid bearer token.'
        });
      }

      const task = getTaskForUser(params.id, currentUser.id);

      if (!task) {
        return status(404, {
          message: 'Task not found.'
        });
      }

      const updatedTask: TaskRecord = {
        ...task,
        title: body.title?.trim() ?? task.title,
        description:
          body.description === undefined
            ? task.description
            : body.description === null
              ? null
              : body.description.trim(),
        completed: body.completed ?? task.completed,
        updatedAt: new Date().toISOString()
      };

      tasks.set(task.id, updatedTask);

      return {
        message: 'Task updated.',
        task: updatedTask
      };
    },
    { body: updateTaskBody, params: taskParams }
  )
  .delete(
    '/tasks/:id',
    ({ params, headers, status }) => {
      const currentUser = getCurrentUser(headers.authorization);

      if (!currentUser) {
        return status(401, {
          message: 'Missing or invalid bearer token.'
        });
      }

      const task = getTaskForUser(params.id, currentUser.id);

      if (!task) {
        return status(404, {
          message: 'Task not found.'
        });
      }

      tasks.delete(task.id);

      return {
        message: 'Task deleted.'
      };
    },
    { params: taskParams }
  );

if (import.meta.main) {
  const port = Number(Bun.env.PORT ?? 3000);

  app.listen(port);
  console.log(`login-app/rest-api listening on http://localhost:${port}`);
}
