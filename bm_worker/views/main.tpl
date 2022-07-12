<!DOCTYPE html>
<html>

    <head>
        <link rel="stylesheet" href="/css/umi_main.css">
        <meta charset="utf-8">
        <title>人脸车辆识别</title>
        <script>
            window.routerBase = "/";
        </script>
    </head>

    <body>
        <script>
        window.CAMERA_LIMIT_COUNT = 16;
        // window.CF_API_PREFIX = 'http://{{ localIp }}:{{ port }}';
        // window.CF_WEBSOCKET = '/ws/track'

        window.CF_API_PREFIX = '';
        window.CF_WEBSOCKET = '/ws/track'

            function back_to_login() {
                window.location.href = "./"
            }
        </script>
        <div id="root"></div>
        <script src="/js/umi_main.js"></script>
    </body>

</html>