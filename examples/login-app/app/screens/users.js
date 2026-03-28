import { UserModal } from './user-modal.js';
import {
    AppButton,
    Badge,
    CardHeader,
    EntityRow,
    ListSection,
    ScreenShell,
    SelectField,
    Surface
} from '../components/index.js';
import { formatDateTime } from '../lib/format.js';
import { matchesUserScope, USER_SCOPE_OPTIONS } from '../lib/options.js';
import {
    closeUserModal,
    ensureUsersLoaded,
    isAuthenticated,
    openUserModal,
    refreshUsers,
    requestUserRemoval,
    saveUserModal,
    setUserFilter,
    setUserFormField,
    state
} from '../state/app-state.js';

function UserRow(user) {
    const isCurrentUser = user.id === state.session.user.id;

    return EntityRow({
        title: user.name,
        subtitle: user.email,
        description: isCurrentUser ? 'Current signed-in account' : `Created ${formatDateTime(user.createdAt)}`,
        badge: Badge({
            variant: isCurrentUser ? 'success' : 'neutral',
            text: isCurrentUser ? 'Current user' : 'Managed account'
        }),
        style: {
            backgroundColor: isCurrentUser ? '#F1FAF4' : '#FFFFFF',
            borderColor: isCurrentUser ? '#B7DCC6' : '#DCCDBE'
        },
        actions: [
            AppButton({
                text: 'Edit',
                size: 'sm',
                variant: 'secondary',
                onClick: () => openUserModal('edit', user)
            }),
            AppButton({
                text: isCurrentUser
                    ? 'Self-delete blocked'
                    : state.loading.deletingUserId === user.id
                      ? 'Deleting...'
                      : 'Delete',
                size: 'sm',
                variant: isCurrentUser ? 'ghost' : 'danger',
                disabled: isCurrentUser || state.loading.deletingUserId === user.id,
                onClick: () => requestUserRemoval(user.id)
            })
        ]
    });
}

function ListHeader(filteredUsers) {
    return View({
        style: {
            width: 'fill',
            flexDirection: 'column',
            gap: 12
        },
        children: [
            Surface({
                backgroundColor: '#FFF8EC',
                borderColor: '#E9D2AA',
                children: [
                    CardHeader({
                        title: 'Manage accounts',
                        subtitle: 'Add a new user or refine the current list view.',
                        actions: [
                            Badge({
                                variant: 'neutral',
                                text: `${filteredUsers.length} visible`
                            })
                        ]
                    }),
                    View({
                        style: {
                            width: 'fill',
                            flexDirection: 'row',
                            gap: 12
                        },
                        children: [
                            AppButton({
                                text: 'Add user',
                                onClick: () => openUserModal('create')
                            }),
                            AppButton({
                                text: 'Refresh',
                                variant: 'secondary',
                                onClick: refreshUsers
                            })
                        ]
                    }),
                    SelectField({
                        label: 'Filter',
                        value: state.filters.users,
                        options: USER_SCOPE_OPTIONS,
                        onChange: setUserFilter
                    })
                ]
            }),
            Surface({
                backgroundColor: '#FFFFFF',
                borderColor: '#DCCDBE',
                children: [
                    Badge({
                        variant: 'success',
                        text: `Current account: ${state.session.user.name}`
                    }),
                    Badge({
                        variant: 'warning',
                        text: 'The API blocks self-delete, and the UI mirrors that rule.'
                    })
                ]
            })
        ]
    });
}

export function UsersScreen(route) {
    if (!isAuthenticated()) {
        route.replace('/');
        return ScreenShell({
            title: 'Redirecting',
            subtitle: 'The users route is protected.',
            backgroundColor: '#F7F2E8',
            children: []
        });
    }

    ensureUsersLoaded();

    const filteredUsers = state.users.filter((user) =>
        matchesUserScope(state.filters.users, user, state.session.user.id)
    );

    return ScreenShell({
        title: 'Users',
        subtitle: 'FlatList-based user management with add, edit, and delete actions.',
        backgroundColor: '#F7F2E8',
        children: [
            ListSection({
                data: filteredUsers,
                listStyle: {
                    height: 'fill'
                },
                contentContainerStyle: {
                    gap: 12
                },
                ListHeaderComponent: () => ListHeader(filteredUsers),
                emptyTitle: state.loading.users
                    ? 'Loading users'
                    : state.users.length === 0
                      ? 'No users yet'
                      : 'No users match this filter',
                emptyDescription: state.loading.users
                    ? 'Fetching users from the API.'
                    : state.users.length === 0
                      ? 'Add the first managed account from the panel above.'
                      : 'Try another filter or add a new account.',
                renderItem: ({ item }) => UserRow(item)
            }),
            UserModal({
                visible: state.userModal.visible,
                mode: state.userModal.mode,
                draft: state.forms.user,
                message:
                    state.userModal.visible && state.notice?.tone === 'error' ? state.notice.text : '',
                submitting: state.loading.userSave,
                onClose: closeUserModal,
                onNameChange: (value) => setUserFormField('name', value),
                onEmailChange: (value) => setUserFormField('email', value),
                onPasswordChange: (value) => setUserFormField('password', value),
                onSubmit: saveUserModal
            })
        ]
    });
}
