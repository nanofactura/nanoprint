import { mount } from 'svelte';
import '../../app.css';
import { setUpWindow } from '$lib/window';
import Onboarding from './Onboarding.svelte';

const target = document.getElementById('app');
if (!target) throw new Error('Missing #app element');

setUpWindow();
mount(Onboarding, { target });
