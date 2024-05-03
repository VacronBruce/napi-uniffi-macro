import test from 'ava';
import { sum } from './index.js';
 
test('TestSum', t => {
	t.is(sum(3, 4), 7);
});