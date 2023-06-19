# coding: utf-8

# 1.Rustで作成したplzファイルをPymeshlabでロード
# 2.メッシュを作成
# 3.open3dデータに変換 ⇒ やり方が分からないので中間ファイルを生成(with_mesh.plz)
# 4.視点設定を行い画像を作成 ⇒ カメラの設定方法について追究する必要がある
# 5.pngとして保存


import open3d as o3d
import numpy as np
import copy
import pymeshlab
import sys


def main():
    
    print("main")
    # pcdをplz に変換して保存
    input_filepath = sys.argv[1]
    temp_filepath = sys.argv[2]
    output_filepath =sys.argv[3]

    # 1.Rustで作成したplzファイルをPymeshlabでロード
    
    print("phase:1")
    ms = pymeshlab.MeshSet()
    ms.load_new_mesh(input_filepath)

    # 2.メッシュを作成
    print("phase:2")
    ms.apply_filter("compute_normals_for_point_sets")
    ms.apply_filter("surface_reconstruction_screened_poisson")
    # ms.apply_filter('remove_isolated_pieces_wrt_diameter', mincomponentdiag=50)
    ms.apply_filter("remove_isolated_pieces_wrt_diameter")

    # 3.open3dデータに変換
    
    print("phase:3")
    ms.save_current_mesh(temp_filepath)

    del ms


    # 4.視点設定を行い画像を作成
    # [Open3Dのカメラの取り扱い ViewControl編](https://zenn.dev/fastriver/articles/open3d-camera-view-control)
    # [Open3Dのカメラの取り扱い PinholeCamera編](https://zenn.dev/fastriver/articles/open3d-camera-pinhole)
    # 視点の設定
    print("phase:4")
    mesh = o3d.io.read_triangle_mesh(temp_filepath)
    mesh.compute_vertex_normals()

    vis = o3d.visualization.Visualizer()
    vis.create_window(
        visible=False,
        width=1920,
        height=1080,
        left=50,
        top=50,
    )
    vis.add_geometry(mesh)

    view_control = vis.get_view_control()
    view_control.set_zoom(0.7)
    view_control.set_front([1, 0.3, 1])
    view_control.set_lookat(mesh.get_center())
    view_control.set_up([0, 0, 1])


    vis.poll_events()
    vis.update_renderer()

    vis.capture_screen_image(output_filepath, do_render=True)
    print("phase:end")


def load_and_save_by_mashlab(input_filepath, output_filepath):
    ms = pymeshlab.MeshSet()
    ms.load_new_mesh(input_filepath)
    ms.save_current_mesh(output_filepath)


def load_and_save_by_o3(input_filepath, output_filepath):
    pcd = o3d.io.read_point_cloud(input_filepath)
    o3d.io.write_point_cloud(output_filepath, pcd)


if __name__ == "__main__":
    if len(sys.argv) != 4:
        print("引数が正しくない")
        quit()
    main()
    # print(sys.argv[1],sys.argv[2],sys.argv[3])