#!/bin/bash

# GPU Worker Test Script
set -e

BASE_URL="http://localhost:8080"
TEST_DIR="test_files"

echo "=== GPU Worker Test Script ==="
echo

# Create test directory
mkdir -p "$TEST_DIR"

# Function to check if service is running
check_service() {
    echo "Checking if service is running..."
    if curl -s "$BASE_URL/health" > /dev/null; then
        echo "✅ Service is running"
    else
        echo "❌ Service is not running. Please start it with 'cargo run'"
        exit 1
    fi
}

# Function to test health endpoint
test_health() {
    echo
    echo "Testing health endpoint..."
    response=$(curl -s "$BASE_URL/health")
    echo "Response: $response"

    if echo "$response" | grep -q "healthy"; then
        echo "✅ Health check passed"
    else
        echo "❌ Health check failed"
        exit 1
    fi
}

# Function to create a simple test GIF
create_test_gif() {
    echo
    echo "Creating test GIF..."

    # Check if ImageMagick is available
    if command -v magick >/dev/null 2>&1; then
        # Create a simple animated GIF with ImageMagick
        magick -size 100x100 xc:red "$TEST_DIR/frame1.png"
        magick -size 100x100 xc:blue "$TEST_DIR/frame2.png"
        magick -size 100x100 xc:green "$TEST_DIR/frame3.png"

        magick -delay 50 "$TEST_DIR/frame1.png" "$TEST_DIR/frame2.png" "$TEST_DIR/frame3.png" -loop 0 "$TEST_DIR/test.gif"

        # Clean up temporary frames
        rm "$TEST_DIR/frame1.png" "$TEST_DIR/frame2.png" "$TEST_DIR/frame3.png"

        echo "✅ Test GIF created at $TEST_DIR/test.gif"
    else
        echo "⚠️  ImageMagick not found. Please provide a test GIF file manually at $TEST_DIR/test.gif"
        echo "   Or install ImageMagick: brew install imagemagick (macOS) or apt-get install imagemagick (Ubuntu)"

        # Check if user provided a GIF
        if [ ! -f "$TEST_DIR/test.gif" ]; then
            echo "❌ No test GIF available. Skipping mirror test."
            return 1
        fi
    fi
    return 0
}

# Function to test mirror endpoint
test_mirror() {
    echo
    echo "Testing GIF mirror endpoint..."

    if [ ! -f "$TEST_DIR/test.gif" ]; then
        echo "❌ Test GIF not found"
        return 1
    fi

    echo "Uploading GIF for mirroring..."

    # Test the mirror endpoint
    if curl -X POST \
        -F "file=@$TEST_DIR/test.gif" \
        "$BASE_URL/mirror-gif" \
        --output "$TEST_DIR/mirrored.gif" \
        --silent \
        --show-error; then

        if [ -f "$TEST_DIR/mirrored.gif" ] && [ -s "$TEST_DIR/mirrored.gif" ]; then
            echo "✅ GIF mirroring successful"
            echo "   Original: $TEST_DIR/test.gif"
            echo "   Mirrored: $TEST_DIR/mirrored.gif"
        else
            echo "❌ Mirrored GIF is empty or not created"
            return 1
        fi
    else
        echo "❌ GIF mirroring failed"
        return 1
    fi
}

# Function to test error handling
test_error_handling() {
    echo
    echo "Testing error handling..."

    # Test with invalid file
    echo "Testing with non-GIF file..."
    echo "This is not a GIF" > "$TEST_DIR/invalid.txt"

    response=$(curl -X POST \
        -F "file=@$TEST_DIR/invalid.txt" \
        "$BASE_URL/mirror-gif" \
        --silent \
        --write-out "HTTP_STATUS:%{http_code}")

    http_status=$(echo "$response" | grep -o "HTTP_STATUS:[0-9]*" | cut -d: -f2)

    if [ "$http_status" -ge 400 ] && [ "$http_status" -lt 500 ]; then
        echo "✅ Error handling works correctly (HTTP $http_status)"
    else
        echo "❌ Error handling failed (HTTP $http_status)"
    fi

    rm "$TEST_DIR/invalid.txt"
}

# Function to benchmark performance
benchmark_performance() {
    echo
    echo "Running performance benchmark..."

    if [ ! -f "$TEST_DIR/test.gif" ]; then
        echo "❌ Test GIF not found for benchmark"
        return 1
    fi

    echo "Performing 5 mirror operations..."

    start_time=$(date +%s.%N)

    for i in {1..5}; do
        echo -n "  Request $i... "
        if curl -X POST \
            -F "file=@$TEST_DIR/test.gif" \
            "$BASE_URL/mirror-gif" \
            --output "$TEST_DIR/benchmark_$i.gif" \
            --silent \
            --show-error; then
            echo "✅"
        else
            echo "❌"
        fi
    done

    end_time=$(date +%s.%N)
    duration=$(echo "$end_time - $start_time" | bc -l)
    avg_time=$(echo "$duration / 5" | bc -l)

    echo "📊 Benchmark Results:"
    echo "   Total time: ${duration}s"
    echo "   Average per request: ${avg_time}s"

    # Clean up benchmark files
    rm -f "$TEST_DIR/benchmark_"*.gif
}

# Main execution
main() {
    check_service
    test_health

    if create_test_gif; then
        test_mirror
        test_error_handling
        benchmark_performance
    fi

    echo
    echo "=== Test Summary ==="
    echo "✅ All tests completed"
    echo "📁 Test files are in: $TEST_DIR/"
    echo
    echo "Manual testing:"
    echo "  curl -X POST -F \"file=@your_image.gif\" $BASE_URL/mirror-gif --output mirrored.gif"
}

# Check for bc command (needed for benchmarks)
if ! command -v bc >/dev/null 2>&1; then
    echo "⚠️  'bc' command not found. Install it for benchmark calculations."
    echo "   macOS: brew install bc"
    echo "   Ubuntu: apt-get install bc"
fi

# Run main function
main
