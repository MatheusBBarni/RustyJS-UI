export let counter = 0;

export function nextSharedCount() {
    counter += 1;
    return counter;
}
