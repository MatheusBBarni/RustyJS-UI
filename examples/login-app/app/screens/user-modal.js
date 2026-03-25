import { Dialog, DialogActions, FieldError, FieldHint, TextField } from '../components/index.js';

export function UserModal(props = {}) {
    return Dialog({
        visible: Boolean(props.visible),
        onRequestClose: props.onClose,
        backdropColor: '#00000066',
        width: 480,
        title: props.mode === 'create' ? 'Add user' : 'Edit user',
        subtitle: 'Manage users from the protected users route.',
        children: [
            ...(props.message
                ? [
                    FieldError({
                        text: props.message
                    })
                ]
                : []),
            TextField({
                label: 'Name',
                value: props.draft.name,
                placeholder: 'User name',
                onChange: props.onNameChange
            }),
            TextField({
                label: 'Email',
                value: props.draft.email,
                placeholder: 'user@example.com',
                onChange: props.onEmailChange
            }),
            TextField({
                label: props.mode === 'create' ? 'Password' : 'Password (leave blank to keep current)',
                value: props.draft.password,
                placeholder: props.mode === 'create' ? 'Password' : 'Leave blank',
                onChange: props.onPasswordChange
            }),
            FieldHint({
                text: 'Self-delete is blocked by the API and mirrored in the UI.'
            }),
            DialogActions({
                cancelText: 'Cancel',
                onCancel: props.onClose,
                confirmText: props.mode === 'create' ? 'Create user' : 'Save user',
                confirmVariant: 'primary',
                confirmDisabled: Boolean(props.submitting),
                onConfirm: props.onSubmit
            })
        ]
    });
}
