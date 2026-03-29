let tab = 'overview';

function AppLayout() {
    return View({
        style: {
            width: 'fill',
            height: 'fill',
            padding: 24,
            direction: 'column',
            gap: 20
        },
        children: [
            Text({
                text: 'Tabs Example',
                style: {
                    fontSize: 28
                }
            }),
            Tabs({
                value: tab,
                onChange: (nextValue) => {
                    tab = nextValue;
                    App.requestRender();
                },
                tabs: [
                    {
                        label: 'Overview',
                        value: 'overview',
                        content: Text({ text: 'Overview panel content' })
                    },
                    {
                        label: 'Settings',
                        value: 'settings',
                        content: Text({ text: 'Settings panel content' })
                    }
                ]
            })
        ]
    });
}

App.run({
    title: 'Tabs Example',
    windowSize: { width: 620, height: 360 },
    render: AppLayout
});
