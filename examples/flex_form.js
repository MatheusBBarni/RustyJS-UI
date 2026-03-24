let email = '';
let name = '';
let role = '';
let resultMessage = 'Saved profile will appear here.';

const roles = [
    { label: 'Engineer', value: 'engineer' },
    { label: 'Designer', value: 'designer' },
    { label: 'Manager', value: 'manager' }
];

function getRoleLabel(value) {
    const selected = roles.find((option) => option.value === value);
    return selected ? selected.label : 'No role selected';
}

function handleEmailChange(nextValue) {
    email = nextValue;
    App.requestRender();
}

function handleNameChange(nextValue) {
    name = nextValue;
    App.requestRender();
}

function handleRoleChange(nextValue) {
    role = nextValue;
    App.requestRender();
}

function handleSave() {
    resultMessage = `Saved ${name || 'Unknown'} (${email || 'no email'}) as ${getRoleLabel(role)}.`;
    App.requestRender();
}

function AppLayout() {
    return View({
        style: {
            width: 'fill',
            height: 'fill',
            padding: 32,
            flexDirection: 'column',
            justifyContent: 'center',
            alignItems: 'center',
            backgroundColor: '#F3F6FA'
        },
        children: [
            View({
                style: {
                    width: 420,
                    padding: { x: 24, y: 20 },
                    flexDirection: 'column',
                    gap: 14,
                    justifyContent: 'flex-start',
                    alignItems: 'center',
                    backgroundColor: '#FFFFFF',
                    borderWidth: 1,
                    borderRadius: 18,
                    borderColor: '#D5DEE8'
                },
                children: [
                    Text({
                        text: 'Profile Form',
                        style: {
                            fontSize: 28,
                            color: '#102033'
                        }
                    }),
                    TextInput({
                        value: email,
                        placeholder: 'Email',
                        onChange: handleEmailChange,
                        style: {
                            width: 320,
                            padding: 10,
                            borderWidth: 1,
                            borderRadius: 10,
                            borderColor: '#C4D0DD',
                            backgroundColor: '#FFFFFF',
                            color: '#102033'
                        }
                    }),
                    TextInput({
                        value: name,
                        placeholder: 'Name',
                        onChange: handleNameChange,
                        style: {
                            width: 320,
                            padding: 10,
                            borderWidth: 1,
                            borderRadius: 10,
                            borderColor: '#C4D0DD',
                            backgroundColor: '#FFFFFF',
                            color: '#102033'
                        }
                    }),
                    SelectInput({
                        value: role,
                        placeholder: 'Role',
                        options: roles,
                        onChange: handleRoleChange,
                        style: {
                            width: 320,
                            padding: 10,
                            borderWidth: 1,
                            borderRadius: 10,
                            borderColor: '#C4D0DD',
                            backgroundColor: '#FFFFFF',
                            color: '#102033'
                        }
                    }),
                    Button({
                        text: 'Save',
                        onClick: handleSave,
                        style: {
                            width: 320,
                            padding: { x: 12, y: 12 },
                            borderRadius: 10,
                            backgroundColor: '#215B9A',
                            color: '#FFFFFF'
                        }
                    }),
                    Text({
                        text: resultMessage,
                        style: {
                            fontSize: 16,
                            color: '#425466'
                        }
                    })
                ]
            })
        ]
    });
}

App.run({
    title: 'Flex Form Example',
    windowSize: { width: 760, height: 560 },
    render: AppLayout
});
