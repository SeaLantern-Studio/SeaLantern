
set -e

UPSTREAM="SeaLantern-Studio/SeaLantern"
BRANCH="beta"

echo "[1/4] 添加 upstream remote..."
git remote remove upstream 2>/dev/null || true
git remote add upstream "https://github.com/$UPSTREAM.git"

echo "[2/4] 拉取上游 $UPSTREAM/$BRANCH ..."
git fetch upstream "$BRANCH"

echo "[3/4] 重置本地 $BRANCH 为 upstream/$BRANCH ..."
git checkout "$BRANCH" 2>/dev/null || true
git reset --hard "upstream/$BRANCH"

echo "[4/4] 强制推送到 origin/$BRANCH ..."
git push origin "$BRANCH" --force

echo ""
echo "同步完成: $UPSTREAM/$BRANCH -> origin/$BRANCH"
