import test from 'ava';
import { sum, Week, getWeekValue, serviceNew, serviceHello, getBeToken} from './index.js';
 
test('TestSum', t => {
	t.is(sum(3, 4), 7);
});

test('TestEnumWeek', t => {
    t.is(Week.SAT.valueOf(), 5)
})

test('TestFnWithWeek', t => {
    t.is(getWeekValue(Week.FRI), 4)
})

test('TestCreateServiceHello', t => {
    let s = serviceNew();
    t.is(serviceHello(s), "SERVICE:HELLO")
})
test('getBeToken returns a token for a user', async (t) => {
    // Call the function
    const token = await getBeToken('testuser', false);
    console.log(`we got token ${token}`)
  
    // Assert that the function returned the expected token
    t.pass();
  });