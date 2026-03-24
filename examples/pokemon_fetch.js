const POKEAPI_BASE_URL =
    globalThis.__POKEAPI_BASE_URL__ || 'https://pokeapi.co/api/v2/pokemon';

let pokemonName = '';
let loading = false;
let errorMessage = '';
let pokemon = null;

function titleize(value) {
    return String(value || '')
        .split('-')
        .filter(Boolean)
        .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
        .join(' ');
}

function buildPokemonSummary(payload) {
    const types = (payload.types || [])
        .slice()
        .sort((left, right) => (left.slot || 0) - (right.slot || 0))
        .map((entry) => titleize(entry.type?.name))
        .filter(Boolean);
    const abilities = (payload.abilities || [])
        .map((entry) => titleize(entry.ability?.name))
        .filter(Boolean)
        .slice(0, 3);

    return {
        name: titleize(payload.name),
        id: payload.id ?? 'Unknown',
        baseExperience: payload.base_experience ?? 'Unknown',
        types: types.length > 0 ? types.join(', ') : 'Unknown',
        abilities: abilities.length > 0 ? abilities.join(', ') : 'Unknown'
    };
}

function handlePokemonNameChange(nextValue) {
    pokemonName = nextValue;
    App.requestRender();
}

async function searchPokemon() {
    const normalizedName = pokemonName.trim().toLowerCase();

    if (!normalizedName) {
        pokemon = null;
        errorMessage = 'Type a Pokemon name before searching.';
        App.requestRender();
        return;
    }

    loading = true;
    errorMessage = '';
    App.requestRender();

    try {
        const responseText = await fetch(
            `${POKEAPI_BASE_URL}/${encodeURIComponent(normalizedName)}`
        );
        const payload = JSON.parse(responseText);
        pokemon = buildPokemonSummary(payload);
    } catch (error) {
        pokemon = null;
        errorMessage = `Could not load "${normalizedName}". ${error.message || 'Try another Pokemon.'}`;
    } finally {
        loading = false;
        App.requestRender();
    }
}

function DetailRow(label, value) {
    return View({
        style: {
            width: 'fill',
            flexDirection: 'column',
            gap: 4,
            padding: { x: 14, y: 12 },
            backgroundColor: '#FFF8E8',
            borderWidth: 1,
            borderRadius: 12,
            borderColor: '#F0D78A'
        },
        children: [
            Text({
                text: label,
                style: {
                    fontSize: 14,
                    color: '#8A6A16'
                }
            }),
            Text({
                text: String(value),
                style: {
                    fontSize: 18,
                    color: '#2B2620'
                }
            })
        ]
    });
}

function renderResultCard() {
    if (loading) {
        return View({
            style: {
                width: 'fill',
                padding: 18,
                borderWidth: 1,
                borderRadius: 18,
                borderColor: '#C7D7EF',
                backgroundColor: '#F6FAFF',
                alignItems: 'center'
            },
            children: [
                Text({
                    text: 'Searching PokeAPI...',
                    style: {
                        fontSize: 20,
                        color: '#2A4A76'
                    }
                })
            ]
        });
    }

    if (errorMessage) {
        return View({
            style: {
                width: 'fill',
                padding: 18,
                borderWidth: 1,
                borderRadius: 18,
                borderColor: '#E2A5A5',
                backgroundColor: '#FFF1F1',
                gap: 6
            },
            children: [
                Text({
                    text: 'Search failed',
                    style: {
                        fontSize: 20,
                        color: '#9F2F2F'
                    }
                }),
                Text({
                    text: errorMessage,
                    style: {
                        fontSize: 16,
                        color: '#7D3E3E'
                    }
                })
            ]
        });
    }

    if (!pokemon) {
        return View({
            style: {
                width: 'fill',
                padding: 18,
                borderWidth: 1,
                borderRadius: 18,
                borderColor: '#D8E1ED',
                backgroundColor: '#FFFFFF',
                gap: 6
            },
            children: [
                Text({
                    text: 'Pokédex card',
                    style: {
                        fontSize: 20,
                        color: '#17324D'
                    }
                }),
                Text({
                    text: 'Search for a Pokemon to load its basic Pokédex details.',
                    style: {
                        fontSize: 16,
                        color: '#56697E'
                    }
                })
            ]
        });
    }

    return View({
        style: {
            width: 'fill',
            flexDirection: 'column',
            gap: 12,
            padding: 18,
            borderWidth: 1,
            borderRadius: 18,
            borderColor: '#D8E1ED',
            backgroundColor: '#FFFFFF'
        },
        children: [
            Text({
                text: pokemon.name,
                style: {
                    fontSize: 28,
                    color: '#17324D'
                }
            }),
            DetailRow('Dex ID', `#${pokemon.id}`),
            DetailRow('Types', pokemon.types),
            DetailRow('Base Experience', pokemon.baseExperience),
            DetailRow('Abilities', pokemon.abilities)
        ]
    });
}

function AppLayout() {
    return View({
        style: {
            width: 'fill',
            height: 'fill',
            padding: 28,
            flexDirection: 'column',
            gap: 18,
            backgroundColor: '#F4F7FB'
        },
        children: [
            View({
                style: {
                    flexDirection: 'column',
                    gap: 6
                },
                children: [
                    Text({
                        text: 'Pokemon Fetch Example',
                        style: {
                            fontSize: 32,
                            color: '#102033'
                        }
                    }),
                    Text({
                        text: 'Search a Pokemon name and load a few details from PokeAPI.',
                        style: {
                            fontSize: 17,
                            color: '#5A6C7F'
                        }
                    })
                ]
            }),
            View({
                style: {
                    width: 'fill',
                    flexDirection: 'row',
                    gap: 12,
                    alignItems: 'center'
                },
                children: [
                    TextInput({
                        value: pokemonName,
                        placeholder: 'Pokemon name',
                        onChange: handlePokemonNameChange,
                        style: {
                            width: 260,
                            padding: 12,
                            borderWidth: 1,
                            borderRadius: 12,
                            borderColor: '#C4D0DD',
                            backgroundColor: '#FFFFFF',
                            color: '#102033'
                        }
                    }),
                    Button({
                        text: loading ? 'Searching...' : 'Search',
                        onClick: searchPokemon,
                        style: {
                            padding: { x: 18, y: 12 },
                            borderRadius: 12,
                            backgroundColor: '#E04545',
                            color: '#FFFFFF'
                        }
                    })
                ]
            }),
            renderResultCard()
        ]
    });
}

App.run({
    title: 'Pokemon Fetch Example',
    windowSize: { width: 760, height: 620 },
    render: AppLayout
});
