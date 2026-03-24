import { NavButton, PageShell } from './screens.js';

function HomeScreen(route) {
    return PageShell({
        title: 'Router Demo',
        children: [
            Text({
                text: `Current path: ${route.path}`,
                style: {
                    fontSize: 17,
                    color: '#425466'
                }
            }),
            Text({
                text: 'This page uses App.createRouter, route params, query parsing, and JS-only history.',
                style: {
                    fontSize: 16,
                    color: '#56697E'
                }
            }),
            View({
                style: {
                    width: 'fill',
                    flexDirection: 'column',
                    gap: 12
                },
                children: [
                    NavButton({
                        text: 'Open project alpha',
                        onClick: () => route.navigate('/projects/alpha?tab=overview')
                    }),
                    NavButton({
                        text: 'Open project beta',
                        onClick: () => route.navigate('/projects/beta?tab=activity')
                    }),
                    NavButton({
                        text: 'Replace with beta',
                        backgroundColor: '#5D6B7A',
                        onClick: () => route.replace('/projects/beta?tab=overview')
                    })
                ]
            })
        ]
    });
}

function ProjectScreen(route) {
    return PageShell({
        title: `Project ${route.params.projectId}`,
        children: [
            Text({
                text: `Path param projectId = ${route.params.projectId}`,
                style: {
                    fontSize: 17,
                    color: '#425466'
                }
            }),
            Text({
                text: `Query tab = ${route.query.tab || 'none'}`,
                style: {
                    fontSize: 16,
                    color: '#56697E'
                }
            }),
            Text({
                text: `Full path: ${route.path}`,
                style: {
                    fontSize: 16,
                    color: '#56697E'
                }
            }),
            View({
                style: {
                    width: 'fill',
                    flexDirection: 'column',
                    gap: 12
                },
                children: [
                    NavButton({
                        text: 'Go to home',
                        onClick: () => route.navigate('/')
                    }),
                    NavButton({
                        text: 'Open activity tab',
                        onClick: () => route.navigate(`/projects/${route.params.projectId}?tab=activity`)
                    }),
                    NavButton({
                        text: 'Back',
                        backgroundColor: '#5D6B7A',
                        onClick: () => route.back()
                    }),
                    NavButton({
                        text: 'Forward',
                        backgroundColor: '#5D6B7A',
                        onClick: () => route.forward()
                    })
                ]
            })
        ]
    });
}

function NotFoundScreen(route) {
    return PageShell({
        title: 'Page not found',
        children: [
            Text({
                text: `No route matched ${route.path}.`,
                style: {
                    fontSize: 16,
                    color: '#7A1F1F'
                }
            }),
            NavButton({
                text: 'Return home',
                onClick: () => route.navigate('/')
            })
        ]
    });
}

const router = App.createRouter({
    initialPath: '/',
    routes: [
        {
            path: '/',
            render: HomeScreen
        },
        {
            path: '/projects/:projectId',
            render: ProjectScreen
        }
    ],
    notFound: NotFoundScreen
});

function AppLayout() {
    return View({
        style: {
            width: 'fill',
            height: 'fill',
            padding: 28,
            backgroundColor: '#EEF3F7',
            flexDirection: 'column',
            gap: 14
        },
        children: [
            Text({
                text: `Router state path: ${router.getPath()}`,
                style: {
                    fontSize: 15,
                    color: '#4F6275'
                }
            }),
            router.render()
        ]
    });
}

App.run({
    title: 'Router Demo Example',
    windowSize: { width: 840, height: 620 },
    render: AppLayout
});
