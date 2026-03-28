import { App, View, Text, Button } from 'RustyJS-UI';

function PackageImportScreen() {
    return View({
        style: {
            direction: 'column',
            gap: 12,
            padding: 20
        },
        children: [
            Text({ text: 'Package import fixture' }),
            Button({ text: 'Save package import' })
        ]
    });
}

App.run({
    title: 'Package Import Fixture',
    windowSize: { width: 480, height: 320 },
    render: PackageImportScreen
});
