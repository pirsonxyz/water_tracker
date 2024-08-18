const SERVER_URL = "http://localhost:3000";

async function sendWater() {
  const water_intake = document.getElementById("water_intake").value;
  const target = document.getElementById("target").value;
  const server_response_div = document.getElementById("serverResponse");

  try {
    console.log("Sending data:", { water_intake, target });
    const response = await fetch(`${SERVER_URL}/add_water`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        water_intake: parseInt(water_intake, 10),
        target: parseInt(target, 10),
      }),
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const server_response = await response.json();
    console.log("Server response:", server_response);
    server_response_div.innerHTML = JSON.stringify(server_response);
  } catch (error) {
    console.error("Error in sendWater:", error);
    server_response_div.innerHTML = "An error occurred. Is server running?";
  }
}

async function updateWater() {
  const water_intake = document.getElementById("water_intake_update").value;
  console.log("Updating water intake:", water_intake);
  const response = await fetch(`${SERVER_URL}/update_water`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      water_intake: parseInt(water_intake, 10),
    }),
  });
}

async function viewWater() {
  const waterdiv = document.getElementById("waterView");
  try {
    const response = await fetch(`${SERVER_URL}/view_water`);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    const text = await response.text();
    waterdiv.innerHTML = `<p>${text}</p>`;
  } catch (error) {
    console.error("Error in viewWater:", error);
    waterdiv.innerHTML = "An error occurred while fetching water data.";
  }
}
