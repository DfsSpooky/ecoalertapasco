document.addEventListener('DOMContentLoaded', function() {
    // Buscar los campos de latitud y longitud en el formulario de Django Admin
    const latInput = document.getElementById('id_latitude');
    const lngInput = document.getElementById('id_longitude');

    if (!latInput || !lngInput) return;

    // Crear el contenedor del mapa
    const mapDiv = document.createElement('div');
    mapDiv.id = 'admin-map';
    mapDiv.style.height = '350px';
    mapDiv.style.width = '95%';
    mapDiv.style.maxWidth = '800px';
    mapDiv.style.marginTop = '10px';
    mapDiv.style.marginBottom = '20px';
    mapDiv.style.borderRadius = '8px';
    mapDiv.style.border = '1px solid #ccc';
    mapDiv.style.clear = 'both';

    // Insertar el contenedor del mapa antes de los campos de coordenadas
    // En Django admin, los campos están dentro de divs con clase 'form-row field-latitude'
    const latRow = latInput.closest('.form-row');
    if (latRow) {
        latRow.parentNode.insertBefore(mapDiv, latRow);
    } else {
        latInput.parentNode.insertBefore(mapDiv, latInput);
    }

    // Coordenadas iniciales (Cerro de Pasco por defecto, o las existentes si hay)
    let initialLat = parseFloat(latInput.value) || -10.6675;
    let initialLng = parseFloat(lngInput.value) || -76.2567;
    let zoomLevel = latInput.value && lngInput.value ? 16 : 14;

    // Inicializar mapa de Leaflet
    const map = L.map('admin-map').setView([initialLat, initialLng], zoomLevel);

    // Agregar capa de OpenStreetMap
    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
        attribution: '&copy; OpenStreetMap contributors'
    }).addTo(map);

    // Crear marcador arrastrable (draggable)
    const marker = L.marker([initialLat, initialLng], {
        draggable: true
    }).addTo(map);

    // Función para actualizar los inputs cuando se mueve el marcador
    function updateCoords(lat, lng) {
        latInput.value = lat.toFixed(6);
        lngInput.value = lng.toFixed(6);
    }

    // Evento de arrastre del marcador
    marker.on('dragend', function(e) {
        const position = marker.getLatLng();
        updateCoords(position.lat, position.lng);
    });

    // Evento de click en el mapa para mover el marcador
    map.on('click', function(e) {
        marker.setLatLng(e.latlng);
        updateCoords(e.latlng.lat, e.latlng.lng);
    });

    // Evento para actualizar el mapa cuando el usuario escribe en los inputs
    function onInputChange() {
        const lat = parseFloat(latInput.value);
        const lng = parseFloat(lngInput.value);
        if (!isNaN(lat) && !isNaN(lng)) {
            const newPos = new L.LatLng(lat, lng);
            marker.setLatLng(newPos);
            map.panTo(newPos);
        }
    }

    latInput.addEventListener('input', onInputChange);
    lngInput.addEventListener('input', onInputChange);
});
