let counter = 0;

function increment() {
    counter += 1;
    App.requestRender();
}

function AppLayout() {
    return View({
        style: {
            direction: 'column',
            padding: 20,
            spacing: 12,
            alignItems: 'center',
            justifyContent: 'center'
        },
        children: [
            Text({
                text: `Count is: ${counter}`,
                style: {
                    fontSize: 24,
                    color: '#111111'
                }
            }),
            Button({
                text: 'Increment',
                onClick: increment,
                style: {
                    padding: 10,
                    backgroundColor: '#007AFF',
                    borderRadius: 8
                }
            })
        ]
    });
}

App.run({
    title: 'Counter App',
    windowSize: { width: 400, height: 300 },
    render: AppLayout
});
