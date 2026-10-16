import { mount } from 'svelte';
import '../../app.css';
import { setUpWindow } from '$lib/window';
import Settings from './Settings.svelte';

const target = document.getElementById('app');
if (!target) throw new Error('Missing #app element');

setUpWindow();
mount(Settings, { target });
