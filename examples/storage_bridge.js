let value = 'nothing saved yet';

async function savePreference() {
    await Storage.set('theme', 'forest');
    value = (await Storage.get('theme')) ?? 'missing';
    App.requestRender();
}

async function clearPreference() {
    await Storage.clear();
    value = (await Storage.get('theme')) ?? 'nothing saved yet';
    App.requestRender();
}

function AppLayout() {
    return View({
        style: {
            width: 'fill',
            height: 'fill',
            padding: 24,
            direction: 'column',
            gap: 14
        },
        children: [
            Text({
                text: 'Storage Bridge Example',
                style: {
                    fontSize: 28
                }
            }),
            Text({
                text: `Stored value: ${value}`
            }),
            Button({
                text: 'Save theme',
                onClick: savePreference,
                style: {
                    padding: 12,
                    borderRadius: 10,
                    backgroundColor: '#1E7A5F',
                    color: '#FFFFFF'
                }
            }),
            Button({
                text: 'Clear theme',
                onClick: clearPreference,
                style: {
                    padding: 12,
                    borderRadius: 10,
                    backgroundColor: '#DDE6EF',
                    color: '#102033'
                }
            })
        ]
    });
}

App.run({
    title: 'Storage Bridge Example',
    windowSize: { width: 540, height: 320 },
    render: AppLayout
});
