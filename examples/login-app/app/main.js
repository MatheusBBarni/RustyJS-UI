import { Badge, AppButton, CardHeader, Surface } from './components/index.js';
import { LandingScreen } from './screens/landing.js';
import { NotFoundScreen } from './screens/not-found.js';
import { TaskDetailScreen } from './screens/task-detail.js';
import { TasksScreen } from './screens/tasks.js';
import { UsersScreen } from './screens/users.js';
import { clearNotice, isAuthenticated, requestLogout, state } from './state/app-state.js';

const router = App.createRouter({
    initialPath: '/',
    routes: [
        { path: '/', render: LandingScreen },
        { path: '/tasks', render: TasksScreen },
        { path: '/tasks/:taskId', render: TaskDetailScreen },
        { path: '/users', render: UsersScreen }
    ],
    notFound: NotFoundScreen
});

const NOTICE_TONES = {
    info: {
        backgroundColor: '#FFF8EC',
        borderColor: '#F1D29E',
        color: '#7D5600'
    },
    success: {
        backgroundColor: '#EEF8F1',
        borderColor: '#B7DCC6',
        color: '#2C7A4B'
    },
    error: {
        backgroundColor: '#FBE9E9',
        borderColor: '#E2A5A5',
        color: '#9F2F2F'
    }
};

function isActivePath(targetPath) {
    const currentPath = router.getPath().split('?')[0];

    if (targetPath === '/tasks') {
        return currentPath === '/tasks' || currentPath.startsWith('/tasks/');
    }

    return currentPath === targetPath;
}

function NavButton(label, path) {
    return AppButton({
        text: label,
        size: 'sm',
        variant: isActivePath(path) ? 'primary' : 'ghost',
        onClick: () => router.navigate(path)
    });
}

function NoticeBanner() {
    if (!state.notice?.text) {
        return null;
    }

    const tone = NOTICE_TONES[state.notice.tone] || NOTICE_TONES.info;

    return Surface({
        padding: 14,
        backgroundColor: tone.backgroundColor,
        borderColor: tone.borderColor,
        borderRadius: 16,
        children: [
            View({
                style: {
                    width: 'fill',
                    flexDirection: 'column',
                    gap: 12
                },
                children: [
                    View({
                        style: {
                            width: 'fill'
                        },
                        children: [
                            Text({
                                text: state.notice.text,
                                style: {
                                    fontSize: 15,
                                    color: tone.color
                                }
                            })
                        ]
                    }),
                    AppButton({
                        text: 'Dismiss',
                        size: 'sm',
                        variant: 'ghost',
                        onClick: clearNotice
                    })
                ]
            })
        ]
    });
}

function TopBar() {
    const currentUser = state.session.user;
    const authenticated = isAuthenticated();

    const actions = authenticated
        ? [
              Badge({
                  variant: 'success',
                  text: currentUser.name
              })
          ]
        : [
              Badge({
                  variant: 'warning',
                  text: 'Guest session'
              })
          ];

    const navChildren = [NavButton('Home', '/')];

    if (authenticated) {
        navChildren.push(
            NavButton('Tasks', '/tasks'),
            NavButton('Users', '/users'),
            AppButton({
                text: 'Logout',
                size: 'sm',
                variant: 'secondary',
                onClick: () => requestLogout(router)
            })
        );
    }

    return Surface({
        padding: 16,
        backgroundColor: '#FFFDFC',
        borderColor: '#DCCDBE',
        borderRadius: 22,
        children: [
            CardHeader({
                title: 'Harbor Console',
                subtitle: authenticated
                    ? `Signed in as ${currentUser.name} (${currentUser.email})`
                    : 'Router, modal, fetch, FlatList, and SelectInput in one native example.',
                actions
            }),
            View({
                style: {
                    width: 'fill',
                    flexDirection: 'row',
                    alignItems: 'center',
                    gap: 12
                },
                children: navChildren
            })
        ]
    });
}

function AppLayout() {
    return View({
        style: {
            width: 'fill',
            height: 'fill',
            padding: 18,
            flexDirection: 'column',
            gap: 14,
            backgroundColor: '#F2E9DB'
        },
        children: [
            TopBar(),
            ...(state.notice?.text ? [NoticeBanner()] : []),
            View({
                style: {
                    width: 'fill',
                    height: 'fill'
                },
                children: [router.render()]
            })
        ]
    });
}

App.run({
    title: 'Login App Example',
    windowSize: { width: 1320, height: 920 },
    render: AppLayout
});
