let value = '';

const options = [
    { label: 'Rust', value: 'rust' },
    { label: 'JavaScript', value: 'javascript' },
    { label: 'TypeScript', value: 'typescript' }
];

function getSelectedLabel(nextValue) {
    const option = options.find((item) => item.value === nextValue);
    return option ? option.label : 'Nothing selected';
}

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
                text: 'SelectInput Example',
                style: {
                    fontSize: 24,
                    color: '#111111'
                }
            }),
            NativeSelect({
                value,
                placeholder: 'Choose a language',
                options,
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
                text: `Selected label: ${getSelectedLabel(value)}`,
                style: {
                    fontSize: 18,
                    color: '#333333'
                }
            })
        ]
    });
}

App.run({
    title: 'SelectInput Example',
    windowSize: { width: 520, height: 260 },
    render: AppLayout
});
