content = '''@echo off
chcp 65001 >nul
cd /d "C:\\vibecode\\cadcraft"

echo ============================================================
echo CADCraft 완전 자동화: 업스트림 동기화 + 업데이트 + 빌드 + 설치 + 깃푸시
echo ============================================================
echo.

echo [1/8] 업스트림(storytold/cadcraft) 최신화 확인 중...
git fetch upstream
git status -uno | findstr "behind" >nul
if errorlevel 1 (
    echo 이미 최신 상태 (업스트림 변경사항 없음)
    set UPSTREAM_UPDATED=0
) else (
    echo 업스트림에 새로운 커밋 발견! 병합 중...
    git merge upstream/main
    if errorlevel 1 (
        echo 업스트림 병합 실패 (충돌 발생)
        echo 수동으로 충돌 해결 후 다시 실행하세요
        pause
        exit /b 1
    )
    echo 업스트림 병합 완료
    set UPSTREAM_UPDATED=1
)
echo.

echo [2/8] 로컬 변경사항 확인...
git status --short
git diff --stat
echo.

echo [3/8] 커밋 메시지 입력 (엔터시 자동: "chore: auto update YYYYMMDD"):
set /p msg=메시지: 
if "%msg%"=="" set msg=chore: auto update %date:~-10,4%%date:~-5,2%%date:~-2,2%

echo.
echo [4/8] 변경사항 스테이징 및 커밋...
git add .
git commit -m "%msg%"
if errorlevel 1 (
    echo 커밋할 변경사항이 없음 (이미 커밋됨 또는 변경없음)
    echo.
    goto :build
)

:build
echo.
echo [5/8] 릴리스 빌드 중... (최초 5~10분, 이후 30~60초)
echo 잠시만 기다려주세요...
taskkill /F /IM cadcraft.exe /IM cadcraft-cli.exe 2>nul
cargo build --release -p cadcraft -p cadcraft-cli
if errorlevel 1 (
    echo 빌드 실패
    pause
    exit /b 1
)
echo 빌드 성공

echo.
echo [6/6] 로컬 설치...
copy /Y target\release\cadcraft.exe "%LOCALAPPDATA%\CADCraft\"
copy /Y target\release\cadcraft-cli.exe "%LOCALAPPDATA%\CADCraft\"
echo 로컬 설치 완료

echo.
echo [7/7] 내 깃허브(mr-leeeee/cadcraft)에 푸시...
git push fork main
if errorlevel 1 (
    echo 푸시 실패
    echo 원인: 인증 필요(Personal Access Token), 충돌, 또는 네트워크
    echo GitHub Settings > Developer settings > Personal access tokens 확인
    pause
    exit /b 1
)

echo.
echo ============================================================
echo 모든 작업 완료!
echo 로컬: %LOCALAPPDATA%\CADCraft\
echo 내 깃허브: https://github.com/mr-leeeee/cadcraft
echo 업스트림: https://github.com/storytold/cadcraft
echo 버전: 
"C:\vibecode\cadcraft\target\release\cadcraft.exe" --version
echo ============================================================
pause
'''

with open(r'C:\Users\mr_lee\Desktop\CADCraft_자동업데이트_깃푸시.bat', 'wb') as f:
    f.write(content.encode('cp949'))
print('완료: CP949 인코딩으로 저장됨')