import { AuthModal } from './auth-modal.js';
import {
    AppButton,
    Badge,
    CardHeader,
    FieldHint,
    ScreenShell,
    Surface
} from '../components/index.js';
import { getApiBaseUrl } from '../lib/api.js';
import { countLabel } from '../lib/format.js';
import {
    closeAuthModal,
    isAuthenticated,
    openAuthModal,
    setLoginField,
    setRegisterField,
    state,
    submitLogin,
    submitRegister
} from '../state/app-state.js';

function AuthActionRow(route) {
    if (isAuthenticated()) {
        return View({
            style: {
                width: 'fill',
                flexDirection: 'row',
                gap: 12,
                alignItems: 'center'
            },
            children: [
                AppButton({
                    text: 'Open tasks',
                    onClick: () => route.navigate('/tasks')
                }),
                AppButton({
                    text: 'Open users',
                    variant: 'secondary',
                    onClick: () => route.navigate('/users')
                }),
                Badge({
                    variant: 'success',
                    text: `Signed in as ${state.session.user.name}`
                })
            ]
        });
    }

    return View({
        style: {
            width: 'fill',
            flexDirection: 'row',
            gap: 12,
            alignItems: 'center'
        },
        children: [
            AppButton({
                text: 'Login',
                onClick: () => openAuthModal('login')
            }),
            AppButton({
                text: 'Register',
                variant: 'secondary',
                onClick: () => openAuthModal('register')
            }),
            Badge({
                variant: 'warning',
                text: 'Public route'
            })
        ]
    });
}

function FeatureChips() {
    return View({
        style: {
            width: 'fill',
            flexDirection: 'column',
            gap: 10
        },
        children: [
            Badge({
                variant: 'success',
                text: 'Modal login and register'
            }),
            Badge({
                variant: 'neutral',
                text: 'Protected tasks and users routes'
            }),
            Badge({
                variant: 'warning',
                text: 'Task detail route uses params and query mode'
            }),
            Badge({
                variant: 'neutral',
                text: 'FlatList powers both task and user collections'
            })
        ]
    });
}

export function LandingScreen(route) {
    const modalMode = state.authModal || 'login';
    const draft = modalMode === 'register' ? state.forms.register : state.forms.login;

    return ScreenShell({
        title: 'Login app',
        subtitle: 'Start on the public route, then move into protected task and user workflows.',
        backgroundColor: '#F7F2E8',
        children: [
            Surface({
                backgroundColor: '#FFFDFC',
                borderColor: '#DCCDBE',
                children: [
                    CardHeader({
                        title: isAuthenticated() ? `Welcome back, ${state.session.user.name}` : 'Initial route',
                        subtitle: isAuthenticated()
                            ? 'You are already authenticated and can jump straight into the protected routes.'
                            : 'Login and register each open their own native modal.',
                        actions: [
                            Badge({
                                variant: isAuthenticated() ? 'success' : 'warning',
                                text: isAuthenticated() ? 'Authenticated' : 'Guest'
                            })
                        ]
                    }),
                    AuthActionRow(route),
                    FieldHint({
                        text: isAuthenticated()
                            ? `Current account: ${state.session.user.email}`
                            : 'Register once, then the app signs you in and unlocks the protected routes.'
                    })
                ]
            }),
            Surface({
                backgroundColor: '#FFF8EC',
                borderColor: '#E9D2AA',
                children: [
                    CardHeader({
                        title: 'What this example exercises',
                        subtitle: 'The UI deliberately uses every major capability available in the runtime today.'
                    }),
                    FeatureChips()
                ]
            }),
            Surface({
                backgroundColor: '#FFFFFF',
                borderColor: '#DCCDBE',
                children: [
                    CardHeader({
                        title: 'API target',
                        subtitle: 'The app talks to the Bun REST API that lives beside this example.'
                    }),
                    Badge({
                        variant: 'neutral',
                        text: getApiBaseUrl()
                    }),
                    FieldHint({
                        text: `${countLabel(state.tasks.length, 'cached task')} and ${countLabel(state.users.length, 'cached user')} are currently loaded in memory.`
                    })
                ]
            }),
            AuthModal({
                visible: Boolean(state.authModal),
                mode: modalMode,
                draft,
                message: state.authModal && state.notice?.tone === 'error' ? state.notice.text : '',
                submitting: state.loading.auth,
                onClose: closeAuthModal,
                onNameChange: (value) => setRegisterField('name', value),
                onEmailChange: (value) =>
                    modalMode === 'register'
                        ? setRegisterField('email', value)
                        : setLoginField('email', value),
                onPasswordChange: (value) =>
                    modalMode === 'register'
                        ? setRegisterField('password', value)
                        : setLoginField('password', value),
                onSubmit: () =>
                    modalMode === 'register' ? submitRegister(route) : submitLogin(route)
            })
        ]
    });
}
