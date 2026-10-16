import { mount } from 'svelte';
import '../../app.css';
import { setUpWindow } from '$lib/window';
import Permission from './Permission.svelte';

const target = document.getElementById('app');
if (!target) throw new Error('Missing #app element');

setUpWindow();
mount(Permission, { target });
