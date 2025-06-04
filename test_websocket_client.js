const WebSocket = require('ws');

// Connect to the websocket server and create a test document
async function createTestDocument() {
  const docId = '01jpfz6zt1f10r762kyvpe7b3j';
  const wsUrl = `ws://127.0.0.1:8080/ws/${docId}`;
  
  console.log(`Connecting to: ${wsUrl}`);
  
  const ws = new WebSocket(wsUrl);
  
  ws.on('open', function open() {
    console.log('WebSocket connection opened');
    
    // Create a simple Y.js update (simulated)
    // This is a very basic binary update for testing
    const testUpdate = Buffer.from([
      4, 8, 143, 190, 237, 196, 13, 0, 39, 1, 9, 119, 111, 114, 107, 102, 108, 111, 119, 115, 4, 109, 97, 105, 110, 1, 39, 0, 143, 190, 237, 196, 13, 0, 2, 105, 100, 2, 4, 0, 143, 190, 237, 196, 13, 1, 4, 109, 97, 105, 110, 39, 0, 143, 190, 237, 196, 13, 0, 4, 110, 97, 109, 101, 2, 4, 0, 143, 190, 237, 196, 13, 6, 13, 77, 97, 105, 110, 32, 87, 111, 114, 107, 102, 108, 111, 119
    ]);
    
    console.log('Sending test update...');
    ws.send(testUpdate);
    
    // Close after a short delay
    setTimeout(() => {
      console.log('Closing connection');
      ws.close();
    }, 2000);
  });
  
  ws.on('message', function message(data) {
    console.log('Received:', data.length, 'bytes');
  });
  
  ws.on('error', function error(err) {
    console.error('WebSocket error:', err);
  });
  
  ws.on('close', function close() {
    console.log('WebSocket connection closed');
  });
}

createTestDocument().catch(console.error); 