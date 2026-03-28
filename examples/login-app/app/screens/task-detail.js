import {
    AppButton,
    Badge,
    CardHeader,
    EmptyState,
    ScreenShell,
    SelectField,
    Surface,
    TextField
} from '../components/index.js';
import { formatDateTime, summarizeText, taskStatusLabel } from '../lib/format.js';
import { TASK_STATUS_OPTIONS } from '../lib/options.js';
import {
    ensureTaskDetail,
    isAuthenticated,
    loadTaskDetail,
    requestTaskRemoval,
    saveTaskDetail,
    setTaskDetailField,
    state
} from '../state/app-state.js';

function findTask(taskId) {
    if (state.taskDetail && state.taskDetail.id === taskId) {
        return state.taskDetail;
    }

    for (const item of state.tasks) {
        if (item.id === taskId) {
            return item;
        }
    }

    return null;
}

function loadingScreen() {
    return ScreenShell({
        title: 'Task',
        subtitle: 'Loading task details...',
        backgroundColor: '#F7F2E8',
        children: [
            EmptyState({
                title: 'Loading task',
                description: 'Fetching the selected task from the API.'
            })
        ]
    });
}

function missingScreen(route) {
    return ScreenShell({
        title: 'Task not found',
        subtitle: 'The selected task is unavailable.',
        backgroundColor: '#F7F2E8',
        children: [
            EmptyState({
                title: 'Missing task',
                description: 'Return to the tasks list and choose another item.'
            }),
            AppButton({
                text: 'Back to tasks',
                onClick: () => route.navigate('/tasks')
            })
        ]
    });
}

function toggleTask(route, task) {
    const nextValue = task.completed ? 'pending' : 'completed';
    setTaskDetailField('completed', nextValue);
    void saveTaskDetail(route);
}

function resetTaskForm(task, route) {
    setTaskDetailField('title', task.title);
    setTaskDetailField('description', task.description || '');
    setTaskDetailField('completed', task.completed ? 'completed' : 'pending');
    route.replace(`/tasks/${task.id}?mode=view`);
}

function buildSummaryChildren(task) {
    const children = [];
    const badgeRow = [];

    children.push(
        CardHeader({
            title: 'Summary',
            subtitle: `Created ${formatDateTime(task.createdAt)} | Updated ${formatDateTime(task.updatedAt)}`,
            actions: [
                Badge({
                    variant: task.completed ? 'success' : 'warning',
                    text: taskStatusLabel(task.completed)
                })
            ]
        })
    );

    badgeRow.push(
        Badge({
            variant: 'neutral',
            text: `Owner ${state.session.user.name}`
        })
    );
    badgeRow.push(
        Badge({
            variant: 'neutral',
            text: `Task id ${task.id}`
        })
    );

    children.push(
        View({
            style: {
                width: 'fill',
                flexDirection: 'row',
                gap: 10
            },
            children: badgeRow
        })
    );

    children.push(
        Text({
            text: summarizeText(task.description, 180),
            style: {
                fontSize: 14,
                color: '#5E6A74'
            }
        })
    );

    return children;
}

function buildEditorHeader(task, route, isEditMode) {
    return CardHeader({
        title: isEditMode ? 'Edit task' : 'Task contents',
        subtitle: isEditMode
            ? 'Save changes back through the REST API.'
            : 'Switch to edit mode to change fields.',
        actions: [
            AppButton({
                text: isEditMode ? 'View mode' : 'Edit mode',
                size: 'sm',
                variant: 'ghost',
                onClick: () => route.replace(`/tasks/${task.id}?mode=${isEditMode ? 'view' : 'edit'}`)
            })
        ]
    });
}

function buildEditorActions(task, route, isEditMode) {
    const children = [];

    if (isEditMode) {
        children.push(
            AppButton({
                text: state.loading.taskSave ? 'Saving...' : 'Save changes',
                disabled: state.loading.taskSave,
                onClick: () => saveTaskDetail(route)
            })
        );
        children.push(
            AppButton({
                text: 'Cancel',
                variant: 'secondary',
                onClick: () => resetTaskForm(task, route)
            })
        );
    } else {
        children.push(
            AppButton({
                text: task.completed ? 'Mark open' : 'Mark complete',
                size: 'sm',
                variant: 'secondary',
                disabled: state.loading.taskSave,
                onClick: () => toggleTask(route, task)
            })
        );
        children.push(
            AppButton({
                text: 'Refresh',
                size: 'sm',
                variant: 'ghost',
                onClick: () => loadTaskDetail(task.id)
            })
        );
    }

    children.push(
        AppButton({
            text: 'Back',
            size: 'sm',
            variant: 'ghost',
            onClick: () => route.navigate('/tasks')
        })
    );
    children.push(
        AppButton({
            text: state.loading.deletingTaskId === task.id ? 'Deleting...' : 'Delete',
            size: 'sm',
            variant: 'danger',
            disabled: state.loading.deletingTaskId === task.id,
            onClick: () => requestTaskRemoval(task.id, route, '/tasks')
        })
    );

    return children;
}

export function TaskDetailScreen(route) {
    if (!isAuthenticated()) {
        route.replace('/');
        return ScreenShell({
            title: 'Redirecting',
            subtitle: 'The task detail route is protected.',
            backgroundColor: '#F7F2E8',
            children: []
        });
    }

    const taskId = route.params.taskId;
    const isEditMode = Boolean(route.query && route.query.mode === 'edit');

    ensureTaskDetail(taskId);

    const task = findTask(taskId);

    if (!task && state.loading.task) {
        return loadingScreen();
    }

    if (!task) {
        return missingScreen(route);
    }

    const editorChildren = [];

    editorChildren.push(buildEditorHeader(task, route, isEditMode));
    editorChildren.push(
        TextField({
            label: 'Title',
            value: state.forms.taskDetail.title,
            placeholder: 'Task title',
            disabled: !isEditMode,
            onChange: (value) => setTaskDetailField('title', value)
        })
    );
    editorChildren.push(
        TextField({
            label: 'Description',
            value: state.forms.taskDetail.description,
            placeholder: 'Task description',
            multiline: true,
            disabled: !isEditMode,
            onChange: (value) => setTaskDetailField('description', value),
            inputStyle: {
                height: 130
            }
        })
    );
    editorChildren.push(
        SelectField({
            label: 'Status',
            value: state.forms.taskDetail.completed,
            options: TASK_STATUS_OPTIONS,
            disabled: !isEditMode,
            onChange: (value) => setTaskDetailField('completed', value)
        })
    );
    editorChildren.push(
        View({
            style: {
                width: 'fill',
                flexDirection: 'row',
                gap: 12
            },
            children: buildEditorActions(task, route, isEditMode)
        })
    );

    return ScreenShell({
        title: task.title,
        subtitle: 'Task details route with view and edit modes.',
        backgroundColor: '#F7F2E8',
        children: [
            Surface({
                backgroundColor: '#FFFDFC',
                borderColor: '#DCCDBE',
                children: buildSummaryChildren(task)
            }),
            Surface({
                backgroundColor: '#FFFFFF',
                borderColor: '#DCCDBE',
                children: editorChildren
            })
        ]
    });
}
