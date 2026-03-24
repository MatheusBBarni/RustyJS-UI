let value = '';

function handleChange(nextValue) {
    value = nextValue;
    App.requestRender();
}

function AppLayout() {
    return View({
        style: {
            direction: 'column',
            padding: 20,
            spacing: 12
        },
        children: [
            Text({
                text: 'TextInput Example',
                style: {
                    fontSize: 24,
                    color: '#111111'
                }
            }),
            TextInput({
                value,
                placeholder: 'Type something',
                onChange: handleChange,
                style: {
                    width: 320,
                    padding: 10,
                    borderWidth: 1,
                    borderRadius: 8,
                    borderColor: '#C7CDD4'
                }
            }),
            Text({
                text: `Current value: ${value}`,
                style: {
                    fontSize: 18,
                    color: '#333333'
                }
            })
        ]
    });
}

App.run({
    title: 'TextInput Example',
    windowSize: { width: 520, height: 260 },
    render: AppLayout
});
