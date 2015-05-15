(function() {

	var downloadScreen;

	downloadScreen = function() {
	var download_html;
	download_html = "<h2>Choose your platform:</h2>";
	download_html += '<a href="https://dl.dropboxusercontent.com/u/17256312/reforge_windows.zip" type="application/octet-stream" class="small radius button">Windows</a><br/>';
	download_html += '<a href="https://dl.dropboxusercontent.com/u/17256312/reforge_mac.zip" type="application/octet-stream" class="small radius button">OSX</a>';
	$('#download_text').html(download_html);
	return $('#downloadScreen').foundation('reveal', 'open');
	};


	$("#download_button").click(function() {
		downloadScreen();
	});

}).call(this);
