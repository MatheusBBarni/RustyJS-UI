import { Dialog, DialogActions, FieldError, FieldHint, TextField } from '../components/index.js';

export function AuthModal(props = {}) {
    return Dialog({
        visible: Boolean(props.visible),
        onRequestClose: props.onClose,
        backdropColor: '#00000066',
        width: 440,
        title: props.mode === 'register' ? 'Register' : 'Login',
        subtitle:
            props.mode === 'register'
                ? 'Create a new account for the protected routes.'
                : 'Use any account you already created in the app.',
        children: [
            ...(props.message
                ? [
                    FieldError({
                        text: props.message
                    })
                ]
                : []),
            ...(props.mode === 'register'
                ? [
                    TextField({
                        label: 'Name',
                        value: props.draft.name,
                        placeholder: 'Your name',
                        onChange: props.onNameChange
                    })
                ]
                : []),
            TextField({
                label: 'Email',
                value: props.draft.email,
                placeholder: 'name@example.com',
                onChange: props.onEmailChange
            }),
            TextField({
                label: 'Password',
                value: props.draft.password,
                placeholder: 'Password',
                type: 'password',
                onChange: props.onPasswordChange
            }),
            FieldHint({
                text: props.mode === 'register'
                    ? 'Registration also signs you in.'
                    : 'Use the password for an account you already created in the app.'
            }),
            DialogActions({
                cancelText: 'Cancel',
                onCancel: props.onClose,
                confirmText: props.mode === 'register' ? 'Register' : 'Login',
                confirmVariant: 'primary',
                confirmDisabled: Boolean(props.submitting),
                onConfirm: props.onSubmit
            })
        ]
    });
}
