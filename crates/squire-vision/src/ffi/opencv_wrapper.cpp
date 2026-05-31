#include <opencv2/imgproc.hpp>
#include <opencv2/core.hpp>

extern "C" {

int squire_match_template_masked(
    const unsigned char* img_data, int img_w, int img_h, int img_channels,
    const unsigned char* tmpl_data, int tmpl_w, int tmpl_h, int tmpl_channels,
    const unsigned char* mask_data,
    int method,
    double* out_max_val, int* out_max_x, int* out_max_y
) {
    if (!img_data || !tmpl_data || !out_max_val || !out_max_x || !out_max_y) {
        return -1;
    }

    int img_type = (img_channels == 1) ? CV_8UC1 :
                   (img_channels == 3) ? CV_8UC3 : CV_8UC4;
    int tmpl_type = (tmpl_channels == 1) ? CV_8UC1 :
                    (tmpl_channels == 3) ? CV_8UC3 : CV_8UC4;

    cv::Mat img(img_h, img_w, img_type, const_cast<unsigned char*>(img_data));
    cv::Mat tmpl(tmpl_h, tmpl_w, tmpl_type, const_cast<unsigned char*>(tmpl_data));

    cv::Mat img_gray, tmpl_gray;

    if (img_channels == 1) {
        img_gray = img;
    } else if (img_channels == 3) {
        cv::cvtColor(img, img_gray, cv::COLOR_BGR2GRAY);
    } else {
        cv::cvtColor(img, img_gray, cv::COLOR_BGRA2GRAY);
    }

    if (tmpl_channels == 1) {
        tmpl_gray = tmpl;
    } else if (tmpl_channels == 3) {
        cv::cvtColor(tmpl, tmpl_gray, cv::COLOR_BGR2GRAY);
    } else {
        cv::cvtColor(tmpl, tmpl_gray, cv::COLOR_BGRA2GRAY);
    }

    cv::Mat result;

    if (mask_data) {
        cv::Mat mask(tmpl_h, tmpl_w, CV_8UC1, const_cast<unsigned char*>(mask_data));
        cv::matchTemplate(img_gray, tmpl_gray, result, method, mask);
    } else {
        cv::matchTemplate(img_gray, tmpl_gray, result, method);
    }

    double min_val, max_val;
    cv::Point min_loc, max_loc;
    cv::minMaxLoc(result, &min_val, &max_val, &min_loc, &max_loc);

    *out_max_val = max_val;
    *out_max_x = max_loc.x;
    *out_max_y = max_loc.y;

    return 0;
}

} // extern "C"
